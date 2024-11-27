//! Implementation for the `list` command.

use colored::Colorize;

#[derive(clap::Args)]
pub struct Args;

/// Display all of the existing servers.
pub fn run(_: &Args) -> Result<(), anyhow::Error> {
    let servers = axiom::get_servers_dirs()?;
    let count = &servers.len();

    let message = match count {
        0 => "No servers found.".to_string(),
        1 => "Found 1 server:".to_string(),
        _ => format!("Found {count} servers:"),
    };
    println!("{}", message.bold());

    for (i, server) in servers.iter().enumerate() {
        let name = server.file_name();
        let name = name.to_str().expect("server name is not valid unicode");
        println!("  {}. {name}", i + 1);
    }

    Ok(())
}
