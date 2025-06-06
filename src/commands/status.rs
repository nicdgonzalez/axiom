//! Ping a Minecraft server to get basic information about it.

use std::io::{Read, Write};

use anyhow::Context;
use colored::Colorize;

use crate::varint::{self, ReadExt};

#[derive(Debug, clap::Args)]
pub(crate) struct Status {
    /// The IP address or hostname of the target Minecraft server.
    #[arg(long, short = 'H', default_value = "127.0.0.1")]
    hostname: String,

    /// The port number on which the Minecraft server is listening for connections.
    #[arg(long, short = 'p', default_value = "25565")]
    port: u16,
}

impl crate::commands::Run for Status {
    fn run(&self) -> anyhow::Result<()> {
        let server_address = format!("{}:{}", self.hostname, self.port);
        tracing::info!("Connecting to server: {server_address}");
        let mut socket = std::net::TcpStream::connect(&server_address)
            .with_context(|| "Failed to connect to Minecraft server")?;

        let handshake = create_handshake_packet(&self.hostname, self.port)
            .with_context(|| "Failed to create Handshake packet")?;
        socket
            .write_all(&handshake)
            .with_context(|| "Failed to send Handshake packet")?;

        let status_request = create_status_request_packet();
        socket
            .write_all(&status_request)
            .with_context(|| "Failed to send Status Request packet")?;

        let response =
            get_status_response(&mut socket).with_context(|| "Failed to get status response")?;

        let mut stdout = std::io::stdout().lock();

        let motd = response
            .description
            .and_then(|description| Some(description.text))
            .unwrap_or("None".to_owned());

        let players = response
            .players
            .as_ref()
            .and_then(|players| Some(players.online.to_string()))
            .unwrap_or("???".to_owned());

        writeln!(stdout, "{}: {}", "Server Address".bold(), server_address).ok();
        writeln!(stdout, "{}: {}", "MOTD".bold(), motd).ok();
        writeln!(stdout, "{}: {}", "Players Online".bold(), players).ok();

        if let Some(sample) = response.players.and_then(|players| players.sample) {
            for player in sample {
                println!("  {} ({})", player.name, player.id);
            }
        }

        writeln!(stdout, "{}: {}", "Version".bold(), response.version.name).ok();

        Ok(())
    }
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

fn create_handshake_packet(server_address: &str, server_port: u16) -> anyhow::Result<Vec<u8>> {
    let packet_id = varint::encode(0x00);
    let protocol_version = varint::encode(0); // This value is not important for the ping.
    let server_address_length = i32::try_from(server_address.len())
        .and_then(|value| Ok(varint::encode(value)))
        .with_context(|| "Failed to fit server address length in an i32")?;
    let server_port_length = std::mem::size_of_val(&server_port);
    let next_state = varint::encode(1);

    let packet_length = packet_id.len()
        + protocol_version.len()
        + server_address_length.len()
        + server_address.len()
        + server_port_length
        + next_state.len();

    let packet_length_encoded = i32::try_from(packet_length)
        .and_then(|size| Ok(varint::encode(size)))
        .with_context(|| "Failed to fit packet length in an i32")?;

    let capacity = packet_length_encoded.len() + packet_length;

    let mut packet = Vec::with_capacity(capacity);
    packet.extend(packet_length_encoded);
    packet.extend(packet_id);
    packet.extend(protocol_version);
    packet.extend(server_address_length);
    packet.extend(server_address.as_bytes());
    packet.extend(server_port.to_be_bytes());
    packet.extend(next_state);
    tracing::debug!("Handshake packet: {packet:?}");

    Ok(packet)
}

fn create_status_request_packet() -> Vec<u8> {
    let packet_id = varint::encode(0x00);
    let packet_length = packet_id.len(); // This request has no additional data.

    let packet_length_encoded = i32::try_from(packet_length)
        .and_then(|value| Ok(varint::encode(value)))
        .unwrap();

    let capacity = packet_length_encoded.len() + packet_length;

    let mut packet = Vec::with_capacity(capacity);
    packet.extend(packet_length_encoded);
    packet.extend(packet_id);
    tracing::debug!("Status Request packet: {packet:?}");

    packet
}

fn get_status_response(socket: &mut std::net::TcpStream) -> anyhow::Result<StatusResponse> {
    tracing::trace!("Getting Status Response from server...");

    _ = socket.read_varint_i32().map_err(|err| match err {
        varint::ReadVarIntError::Io(inner) => match inner.kind() {
            std::io::ErrorKind::UnexpectedEof => {
                anyhow::anyhow!("No response from server. Are you sure this is a Minecraft server?")
            }
            _ => anyhow::anyhow!("Failed to get packet length"),
        },
        _ => anyhow::anyhow!("{err}"),
    })?;

    let packet_id = socket
        .read_varint_i32()
        .with_context(|| "Failed to get packet ID")?;

    if packet_id != 0x00 {
        return Err(anyhow::anyhow!(
            "Expected the packet ID to be 0, got {packet_id}"
        ));
    }

    let data_length = socket
        .read_varint_i32()
        .with_context(|| "Failed to get data length")?;

    let mut buffer = vec![0u8; data_length as usize];
    socket
        .read_exact(&mut buffer)
        .with_context(|| "Failed to get data")?;

    let data = String::from_utf8(buffer)
        .with_context(|| anyhow::anyhow!("Expected response to be valid UTF-8"))
        .and_then(|content| {
            serde_json::from_str(&content).with_context(|| "Failed to parse JSON status response")
        })?;

    Ok(data)
}
