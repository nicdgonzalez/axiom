//! Implementation for the `delete` command.

use std::io::Write;

use anyhow::Context;
use colored::Colorize;

#[derive(clap::Args)]
pub struct Args {
    /// The unique name used to identify the server.
    pub name: String,
    /// Automatically confirm deletion without requiring user input.
    #[arg(short = 'y', long)]
    pub assume_yes: bool,
}

/// Delete an existing Minecraft server.
pub fn run(args: &Args) -> Result<(), anyhow::Error> {
    let (name, server) = axiom::validate_server_exists(&args.name)?;
    let server_backups =
        axiom::get_server_backups_path(&name).with_context(|| "unable to get backups path")?;

    if !args.assume_yes {
        print!(
            "{} {} (y/N): ",
            "*".cyan(),
            format!("Are you sure you want to delete '{name}'?").bold()
        );
        std::io::stdout().flush()?;

        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;

        if input.trim().to_lowercase() != "y" {
            return Ok(());
        }
    }

    std::fs::remove_dir_all(&server)?;

    if server_backups.exists() {
        std::fs::remove_dir_all(&server_backups)?;
    }

    eprintln!("{}", format!("Server '{name}' has been deleted.").green());
    Ok(())
}
