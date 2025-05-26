//! Implementation of the `status` command.

use std::io::{Read, Write};

use anyhow::Context;
use colored::Colorize;

#[derive(Debug, clap::Args)]
pub(crate) struct StatusCommand {
    #[arg(long, short = 'H', default_value = "127.0.0.1")]
    hostname: String,

    #[arg(long, short = 'p', default_value = "25565")]
    port: u16,
}

#[derive(serde::Deserialize)]
struct StatusResponse {
    description: Option<Description>,
    #[allow(unused)]
    favicon: Option<String>,
    players: Option<Players>,
    version: Version,
}

#[derive(serde::Deserialize)]
struct Description {
    #[allow(unused)]
    color: String,
    text: String,
}

#[derive(serde::Deserialize)]
struct Players {
    #[allow(unused)]
    max: u32,
    online: u32,
    #[allow(unused)]
    sample: Option<Vec<Sample>>,
}

#[derive(serde::Deserialize)]
struct Sample {
    #[allow(unused)]
    name: String,
    #[allow(unused)]
    id: String,
}

#[derive(serde::Deserialize)]
struct Version {
    name: String,
    #[allow(unused)]
    protocol: i32,
}

impl crate::commands::Run for StatusCommand {
    fn run(&self) -> anyhow::Result<()> {
        let server_address = format!("{}:{}", self.hostname, self.port);

        let mut socket = std::net::TcpStream::connect(&server_address)
            .with_context(|| "Failed to connect to server")?;

        send_handshake_packet(&mut socket, &self.hostname, self.port)
            .with_context(|| "Failed to send handshake packet")?;

        send_status_request_packet(&mut socket)
            .with_context(|| "Failed to send status request packet")?;

        let StatusResponse {
            description,
            favicon: _,
            players,
            version,
        } = get_status_response(&mut socket).with_context(|| "Failed to get status response")?;

        let mut stdout = std::io::stdout().lock();

        writeln!(stdout, "{}: {}", "Connected to".bold(), server_address).ok();

        let motd = description
            .and_then(|description| Some(description.text))
            .unwrap_or("None".to_owned());
        writeln!(stdout, "{}: {motd}", "MOTD".bold()).ok();

        let players = players
            .and_then(|players| Some(players.online.to_string()))
            .unwrap_or("???".to_owned());
        writeln!(stdout, "{}: {players}", "Players Online".bold()).ok();

        writeln!(stdout, "{}: {}", "Version".bold(), version.name).ok();

        Ok(())
    }
}

fn get_status_response(socket: &std::net::TcpStream) -> anyhow::Result<StatusResponse> {
    let mut reader = std::io::BufReader::new(socket);
    _ = varint_decode(&mut reader)
        .with_context(|| "Failed to read status response packet length")?;

    let mut buffer = [0u8; 1];
    reader
        .read_exact(&mut buffer)
        .with_context(|| "Failed to get packet ID")?;
    let packet_id = buffer[0];

    if packet_id != 0x00 {
        return Err(anyhow::anyhow!(
            "Expected status response packet ID to be 0, got {packet_id}"
        ));
    }

    let response_length =
        varint_decode(&mut reader).with_context(|| "Failed to read status response length")?;
    let mut buffer = vec![0u8; response_length as usize];
    reader
        .read_exact(&mut buffer)
        .with_context(|| "Failed to read status response")?;

    // Parse the JSON response
    let content = String::from_utf8_lossy(&buffer);
    let data: StatusResponse =
        serde_json::from_str(&content).with_context(|| "Failed to parse status response")?;

    Ok(data)
}

fn send_handshake_packet<W>(writer: &mut W, address: &str, port: u16) -> Result<(), std::io::Error>
where
    W: std::io::Write,
{
    let address_length = varint_encode(address.len() as u32);
    let mut handshake: Vec<u8> = Vec::with_capacity(1 + 1 + address_length.len() + 2 + 1);

    // Packet ID
    // For VarInts with values less than 127 (0x7F), don't use the `varint_encode` function since
    // these values are already in the proper format.
    handshake.push(0x00);

    // Protocol version
    //
    // `110` is the highest valid protocol number under 127. This is a minor optimization to avoid
    // having to encode a value. (NOTE: This value is not important for the ping.)
    handshake.push(110);

    // Server address
    handshake.extend(address_length);
    handshake.extend(address.as_bytes());

    // Server port
    handshake.extend(port.to_be_bytes());

    // Next state
    handshake.push(1);

    let packet_length = varint_encode(handshake.len() as u32);
    let mut buffer = Vec::with_capacity(packet_length.len() + handshake.len());
    _ = buffer.write_all(&packet_length)?;
    _ = buffer.write_all(&handshake)?;

    writer.write_all(&mut buffer)?;
    Ok(())
}

fn send_status_request_packet<W>(writer: &mut W) -> Result<(), std::io::Error>
where
    W: std::io::Write,
{
    // This packet contains the length of the packet ID (1) and data (0), then the packet ID.
    writer.write_all(&[1 + 0, 0x00])?;
    Ok(())
}

fn varint_encode(value: u32) -> Vec<u8> {
    let mut buffer = Vec::with_capacity(4);
    let mut value = value;

    while value > 0b0000_0000_0000_0000_0000_0000_0111_1111 {
        buffer.push(((value & 0b0111_1111) as u8) | 0b1000_0000);
        value >>= 7;
    }

    buffer.push(value as u8);
    assert!(buffer.len() <= 4);
    buffer
}

/// Represents an error that may occur while decoding a VarInt.
#[derive(Debug)]
enum DecodeError {
    ValueTooLarge,
    Io(std::io::Error),
}

impl std::fmt::Display for DecodeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ValueTooLarge => write!(f, "varint is greater than 32 bits"),
            Self::Io(inner) => write!(f, "{inner}"),
        }
    }
}

impl std::error::Error for DecodeError {}

impl From<std::io::Error> for DecodeError {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}

fn varint_decode<R>(reader: &mut R) -> Result<u32, DecodeError>
where
    R: std::io::Read,
{
    let mut value = 0u32;
    let mut position = 0;

    loop {
        let mut buffer = [0u8; 1];
        reader.read_exact(&mut buffer)?;
        let byte = buffer[0];

        value |= ((byte & 0b0111_1111) as u32) << position;
        position += 7;

        // The first bit indicates whether there is more data.
        if byte & 0b1000_0000 == 0 {
            break;
        }

        if position >= 32 {
            return Err(DecodeError::ValueTooLarge);
        }
    }

    Ok(value)
}
