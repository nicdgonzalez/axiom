//! Implementation for the `start` command.

use std::process;

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
    let window_name = format!("axiom:{}", &name);

    // Create tmux session named "axiom" if it does not already exist.
    let session = tmux::Session::new("axiom");

    if !session.exists()? {
        session.create(Some(&server))?;
    }

    // Check if there is already an existing window for this server:
    //
    // TODO: Use tmux::Window when it becomes available.
    if !tmux::Session::new(&window_name).exists()? {
        let status = process::Command::new("tmux")
            .args([
                "new-window",
                "-t",
                "axiom",
                "-n",
                &name,
                "-c",
                &server.to_str().unwrap(),
            ])
            .stdout(process::Stdio::null())
            .stderr(process::Stdio::null())
            .status()?;

        debug_assert!(status.success());
    }

    // Start the server.
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

    // TODO: Use Window::send_keys when it becomes available.
    tmux::send_command(&window_name, &command.join(" "))
        .with_context(|| "failed to start server in tmux session")?;

    eprintln!(
        "{}",
        "Server starting! You should be able to connect soon.".yellow()
    );

    // TODO: Add option to poll `tmux capture-pane` until server says ready.

    Ok(())
}
