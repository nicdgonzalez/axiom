//! Implementation for the `send-command` command.

use anyhow::anyhow;
use colored::Colorize;

#[derive(clap::Args)]
pub struct Args {
    /// The unique name used to identify the server.
    pub name: String,
    /// The command to send to the server.
    pub command: String,
}

/// Send a command to the specified server.
pub fn run(args: &Args) -> Result<(), anyhow::Error> {
    let session_name = format!("axiom_{}", args.name);

    if !axiom::tmux::exists(&session_name)? {
        return Err(anyhow!("tmux session '{}' not found", session_name));
    }

    axiom::tmux::send_command(&session_name, &args.command)?;
    println!("{}", "Command sent successfully!".green());
    Ok(())
}
