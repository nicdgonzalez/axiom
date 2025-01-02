//! Implementation for the `plugin add` command.

use anyhow::anyhow;

#[derive(clap::Args)]
pub struct Args {
    /// The unique name used to identify the server.
    pub name: String,
    /// The name of the new plugin JAR file.
    pub filename: String,
    /// The link to the plugin JAR file.
    #[arg(long, conflicts_with = "file")]
    pub url: String,
    /// The absolute or relative path to a local plugin JAR file.
    #[arg(long)]
    pub file: String,
}

/// Add a new plugin to an existing server.
pub fn run(args: &Args) -> Result<(), anyhow::Error> {
    let (_, server) = axiom::validate_server_exists(&args.name)?;

    // Check if a plugin with the same name already exists.
    let filename = args
        .filename
        .ends_with(".jar")
        .then(|| args.filename.clone())
        .or_else(|| format!("{}.jar", args.filename).into())
        .unwrap();

    let plugin = server.join("plugins").join(filename);

    if plugin.exists() {
        return Err(anyhow!(
            "plugin already exists. use the update command instead"
        ));
    }

    let client = reqwest::blocking::Client::new();
    let response = client.get(&args.url).send()?.text()?;

    Ok(())
}
