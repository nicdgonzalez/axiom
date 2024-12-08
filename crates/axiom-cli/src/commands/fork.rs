//! Implementation for the `fork` command.

#[derive(clap::Args)]
pub struct Args {
    /// The unique name used to identify the server.
    pub source: String,
    pub destination: String,
}

/// Create a new server from an existing server.
pub fn run(_args: &Args) -> Result<(), anyhow::Error> {
    todo!()
}
