use std::io::{Read, Write};
use std::net::ToSocketAddrs;

use anyhow::{Context, anyhow};
use colored::Colorize;

use varint::{self, ReadExt};

use crate::commands::Run;
use crate::config::Config;

#[derive(Debug, clap::Args)]
pub struct Status {
    /// The maximum number of seconds to wait before failing to connect to the server.
    #[arg(long, default_value = "10")]
    pub(crate) timeout: u64,
}

impl Run for Status {
    fn run(&self) -> Result<(), anyhow::Error> {
        let directory = std::env::current_dir().expect("failed to get the current directory");
        let config = Config::from_path(Config::path(directory))
            .with_context(|| "failed to load configuration")?;

        let hostname = config
            .properties
            .as_ref()
            .and_then(|properties| {
                properties
                    .items
                    .get("server-ip")
                    .and_then(|value| value.as_str())
            })
            .unwrap_or_else(|| "127.0.0.1");

        let port = config
            .properties
            .as_ref()
            .and_then(|properties| {
                properties
                    .items
                    .get("server-port")
                    .and_then(|value| value.as_integer())
            })
            .map(|port| u16::try_from(port).with_context(|| "invalid port number"))
            .unwrap_or_else(|| Ok(25565))?;

        let server_address = format!("{}:{}", hostname, port);
        let timeout = std::time::Duration::from_secs(self.timeout);

        tracing::info!("Connecting to server: {server_address}");
        let mut socket = server_address
            .to_socket_addrs()
            .with_context(|| "failed to resolve server address")?
            .find_map(|addr| std::net::TcpStream::connect_timeout(&addr, timeout).ok())
            .with_context(|| "failed to connect to Minecraft server")?;

        send_handshake_packet(&mut socket, &hostname, port)?;
        send_status_request_packet(&mut socket)?;
        let response =
            get_status_response(&mut socket).with_context(|| "failed to get status response")?;

        let mut stdout = std::io::stdout().lock();

        let motd = response
            .description
            .map(|description| description.text)
            .unwrap_or("None".to_owned());

        let players = response
            .players
            .as_ref()
            .map(|players| players.online.to_string())
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

fn send_handshake_packet(
    socket: &mut std::net::TcpStream,
    server_address: &str,
    server_port: u16,
) -> anyhow::Result<()> {
    let handshake = create_handshake_packet(server_address, server_port)
        .with_context(|| "failed to create Handshake packet")?;

    socket
        .write_all(&handshake)
        .with_context(|| "failed to send Handshake packet")
}

/// Construct the Handshake packet.
///
/// https://minecraft.wiki/w/Java_Edition_protocol/Server_List_Ping#Handshake
fn create_handshake_packet(hostname: &str, port: u16) -> anyhow::Result<Vec<u8>> {
    let packet_id = varint::encode(0x00);
    let protocol_version = varint::encode(0); // This value is not important for the ping.
    let server_address_length = i32::try_from(hostname.len())
        .map(varint::encode)
        // The maximum length of a valid hostname is 253.
        // https://en.m.wikipedia.org/wiki/Hostname#Syntax
        .with_context(|| "failed to fit hostname length in an i32")?;
    let server_port_length = std::mem::size_of_val(&port);
    let next_state = varint::encode(1);

    let packet_length = packet_id.len()
        + protocol_version.len()
        + server_address_length.len()
        + hostname.len()
        + server_port_length
        + next_state.len();

    let packet_length_encoded = i32::try_from(packet_length)
        .map(varint::encode)
        .with_context(|| "failed to fit packet length in an i32")?;

    let capacity = packet_length_encoded.len() + packet_length;

    let mut packet = Vec::with_capacity(capacity);
    packet.extend(packet_length_encoded);
    packet.extend(packet_id);
    packet.extend(protocol_version);
    packet.extend(server_address_length);
    packet.extend(hostname.as_bytes());
    packet.extend(port.to_be_bytes());
    packet.extend(next_state);
    tracing::debug!("Handshake packet: {packet:?}");

    Ok(packet)
}

fn send_status_request_packet(socket: &mut std::net::TcpStream) -> anyhow::Result<()> {
    let status_request = create_status_request_packet();

    socket
        .write_all(&status_request)
        .with_context(|| "failed to send Status Request packet")
}

/// Construct the Status Request packet.
///
/// https://minecraft.wiki/w/Java_Edition_protocol/Server_List_Ping#Status_Request
fn create_status_request_packet() -> Vec<u8> {
    let packet_id = varint::encode(0x00);
    let packet_length = packet_id.len(); // This request has no additional data.
    let packet_length_encoded = i32::try_from(packet_length).map(varint::encode).unwrap();
    let capacity = packet_length_encoded.len() + packet_length;

    let mut packet = Vec::with_capacity(capacity);
    packet.extend(packet_length_encoded);
    packet.extend(packet_id);
    tracing::debug!("Status Request packet: {packet:?}");

    packet
}

/// Get and parse the Status Response packet from the server, which returns JSON data containing
/// information about the server (e.g., the Message of the Day (MOTD), online players, etc.).
///
/// https://minecraft.wiki/w/Java_Edition_protocol/Server_List_Ping#Status_Response
fn get_status_response(socket: &mut std::net::TcpStream) -> anyhow::Result<StatusResponse> {
    tracing::trace!("Getting Status Response from server...");

    if let Err(err) = socket.read_varint_i32() {
        if let varint::ReadVarIntError::ReadFailed { source } = &err {
            // Indicates there *is* a server listening to requests at this address,
            // but it probably disregarded our request because it's not a Minecraft server.
            if source
                .downcast_ref::<std::io::Error>()
                .filter(|e| e.kind() == std::io::ErrorKind::UnexpectedEof)
                .is_some()
            {
                return Err(anyhow::anyhow!(
                    "no response from server. are you sure this is a Minecraft server?"
                ));
            }
        }

        return Err(err.into());
    }

    let packet_id = socket
        .read_varint_i32()
        .with_context(|| "failed to get packet ID")?;

    if packet_id != 0x00 {
        return Err(anyhow!("expected the packet ID to be 0, got {packet_id}"));
    }

    let data_length = socket
        .read_varint_i32()
        .with_context(|| "failed to get data length")?;

    let mut buffer = vec![0u8; data_length as usize];
    socket
        .read_exact(&mut buffer)
        .with_context(|| "failed to get data")?;

    let content =
        String::from_utf8(buffer).with_context(|| "expected response to be valid UTF-8")?;

    let data: StatusResponse =
        serde_json::from_str(&content).with_context(|| "failed to parse response body")?;

    Ok(data)
}
