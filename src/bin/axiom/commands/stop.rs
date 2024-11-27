//! Implementation for the `stop` command.

use anyhow::{anyhow, Context};
use colored::Colorize;

#[derive(clap::Args)]
pub struct Args {
    /// The unique name used to identify the server.
    pub name: String,
}

/// Close a server, disconnecting all players.
pub fn run(args: &Args) -> Result<(), anyhow::Error> {
    let (name, _) = axiom::validate_server_exists(&args.name)?;
    let session_name = format!("axiom_{}", &name);

    if axiom::tmux::exists(&session_name)
        .with_context(|| "unable to determine if tmux session exists")?
    {
        axiom::tmux::destroy(&session_name).with_context(|| "failed to destroy tmux session")?;
        println!(
            "{}",
            "Server stopping! Players will be disconnected shortly.".yellow()
        );
    } else {
        return Err(anyhow!("expected server to be running in a tmux session"));
    }

    Ok(())
}
