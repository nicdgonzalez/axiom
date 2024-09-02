pub fn handler() -> anyhow::Result<()> {
    let servers = axiom::server::get_server_dirs()?;
    let count = &servers.len();

    match &count {
        0 => println!("No servers found."),
        1 => println!("Found 1 server:"),
        _ => println!("Found {count} servers:"),
    };

    for (i, server) in servers.iter().enumerate() {
        let name = &server.file_name().into_string().unwrap();
        println!("  {}. {name}", i + 1);
    }

    Ok(())
}
