//! Implementation for the `start` command.

use anyhow::Context;
use colored::Colorize;

#[derive(clap::Args)]
pub struct Args {
    /// The unique name used to identify the server.
    pub name: String,
}

/// Open a server, allowing players to connect to the world.
pub fn run(args: &Args) -> Result<(), anyhow::Error> {
    let (name, server) = axiom::validate_server_exists(&args.name)?;

    let session_name = format!("axiom_{}", &name);
    axiom::tmux::create(&session_name, Some(server))
        .with_context(|| "failed to create tmux session")?;

    let command = [
        "java",
        "-Xms5G",
        "-Xmx5G",
        "-XX:+UseG1GC",
        "-XX:+ParallelRefProcEnabled",
        "-XX:MaxGCPauseMillis=200",
        "-XX:+UnlockExperimentalVMOptions",
        "-XX:+DisableExplicitGC",
        "-XX:+AlwaysPreTouch",
        "-XX:G1NewSizePercent=30",
        "-XX:G1MaxNewSizePercent=40",
        "-XX:G1HeapRegionSize=8M",
        "-XX:G1ReservePercent=20",
        "-XX:G1HeapWastePercent=5",
        "-XX:G1MixedGCCountTarget=4",
        "-XX:InitiatingHeapOccupancyPercent=15",
        "-XX:G1MixedGCLiveThresholdPercent=90",
        "-XX:G1RSetUpdatingPauseTimePercent=5",
        "-XX:SurvivorRatio=32",
        "-XX:+PerfDisableSharedMem",
        "-XX:MaxTenuringThreshold=1",
        "-Dusing.aikars.flags=https://mcflags.emc.gs",
        "-Daikars.new.flags=true",
        "-jar",
        "server.jar",
        "--nogui",
    ];

    axiom::tmux::send_command(&session_name, &command.join(" "))
        .with_context(|| "failed to start server in tmux session")?;

    println!(
        "{}",
        "Server starting! You should be able to connect soon.".yellow()
    );

    Ok(())
}
