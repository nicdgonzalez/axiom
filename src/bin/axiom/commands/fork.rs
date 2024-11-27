//! Implementation for the `fork` command.

use anyhow::{anyhow, Context};
use colored::Colorize;

#[derive(clap::Args)]
pub struct Args {
    /// The unique name used to identify the server.
    pub source: String,
    pub destination: String,
}

/// Create a new server from an existing server.
pub fn run(args: &Args) -> Result<(), anyhow::Error> {
    todo!()
}
