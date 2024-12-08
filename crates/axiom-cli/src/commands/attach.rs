//! Implementation for the `attach` command.

#[derive(clap::Args)]
pub struct Args {
    /// The unique name used to identify the server.
    pub name: String,
}

/// Open the Minecraft server's console.
pub fn run(_args: &Args) -> Result<(), anyhow::Error> {
    todo!()
}
