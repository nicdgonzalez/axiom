//! Implementation for the `list` command.

use colored::Colorize;

#[derive(clap::Args)]
pub struct Args {
    /// The unique name used to identify the server.
    pub name: Option<String>,
}

struct ServerInfo {
    path: String,
    version: String,
    name: String,
}

/// Display all of the existing servers.
pub fn run(_: &Args) -> Result<(), anyhow::Error> {
    let servers = axiom::get_servers_dirs()?;
    let count = &servers.len();

    let message = match count {
        0 => "No servers found.".to_string(),
        1 => "Found 1 server:".to_string(),
        _ => format!("Found {count} servers:"),
    };
    eprintln!("{}", message.bold());

    let server_info: Vec<ServerInfo> = servers.iter().map(|s| get_server_info(&s)).collect();
    let paths: Vec<String> = server_info.iter().map(|e| e.path.to_string()).collect();
    let versions: Vec<String> = server_info.iter().map(|e| e.version.to_string()).collect();
    let names: Vec<String> = server_info.iter().map(|e| e.name.to_string()).collect();

    for server in server_info.iter() {
        let longest_path = get_string_longest_length(&paths);
        let longest_version = get_string_longest_length(&versions);
        let longest_name = get_string_longest_length(&names);
        println!(
            "{:<longest_path$} {:>longest_version$} {:<longest_name$}",
            server.path, server.version, server.name,
        );
    }

    Ok(())
}

fn get_server_info(server: &std::fs::DirEntry) -> ServerInfo {
    let dirpath = server.path();
    let name = server.file_name();
    let version = axiom::get_version_installed(&server.path().join("server.jar"))
        .unwrap_or_else(|| "?".to_string());

    ServerInfo {
        path: dirpath.display().to_string(),
        version,
        name: name
            .to_str()
            .expect("server name is not valid unicode")
            .to_string(),
    }
}

fn get_string_longest_length(it: &[String]) -> usize {
    let mut sizes: Vec<usize> = it.iter().map(|s| s.len()).collect();
    sizes.sort();

    sizes.last().unwrap_or_else(|| &0).to_owned()
}
