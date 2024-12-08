//! Implementation for the `plugin add` command.

#[derive(clap::Args)]
pub struct Args {
    /// The unique name used to identify the server.
    pub name: String,
    /// The link to the plugin JAR file.
    #[arg(long, conflicts_with = "file")]
    pub url: String,
    /// The absolute or relative path to a local plugin JAR file.
    #[arg(long)]
    pub file: String,
}

/// Add a new plugin to an existing server.
pub fn run(_args: &Args) -> Result<(), anyhow::Error> {
    todo!()
}
