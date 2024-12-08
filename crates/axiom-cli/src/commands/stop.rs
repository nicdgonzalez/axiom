//! Implementation for the `stop` command.

use std::process;

use anyhow::anyhow;
use colored::Colorize;

#[derive(clap::Args)]
pub struct Args {
    /// The unique name used to identify the server.
    pub name: String,
}

/// Close a server, disconnecting all players.
pub fn run(args: &Args) -> Result<(), anyhow::Error> {
    let (name, _) = axiom::validate_server_exists(&args.name)?;
    let window_name = format!("axiom:{}", &name);

    // TODO: Implement Window struct in tmux crate.
    let window = tmux::Session::new(&window_name);

    if !window.exists()? {
        return Err(anyhow!("expected server to be running in a tmux session"));
    }

    let status = process::Command::new("tmux")
        .args(["kill-window", "-t", &window.name])
        .stdout(process::Stdio::null())
        .stderr(process::Stdio::null())
        .status()?;

    debug_assert!(status.success());

    eprintln!(
        "{}",
        "Server stopping! Players will be disconnected shortly.".yellow()
    );
    Ok(())
}
