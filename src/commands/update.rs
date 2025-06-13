use std::io::Write;

use anyhow::Context;
use colored::Colorize;

#[derive(Debug, clap::Args)]
pub struct Update {
    /// The version of Minecraft to use.
    version: Option<String>,

    /// Upgrade to the latest version, even if the latest version is not yet stable.
    #[arg(long, short = 'e')]
    allow_experimental: bool,

    /// Downgrade to an older version of Minecraft.
    #[arg(long, short = 'd')]
    allow_downgrade: bool,

    /// Seconds to wait before failing to download the new server JAR.
    #[arg(long, short = 't', default_value = "120")]
    timeout: u64,

    #[clap(flatten)]
    cwd: crate::args::BaseDirectory,
}

impl crate::commands::Run for Update {
    fn run(&self) -> Result<(), anyhow::Error> {
        let versions = paper::versions().with_context(|| "failed to get valid versions")?;

        let version = match &self.version {
            Some(version) => versions
                .iter()
                .find(|&entry| version == entry.as_str())
                .ok_or_else(|| anyhow::anyhow!("invalid version provided"))?,
            None => versions.last().with_context(|| "no versions available")?,
        };

        // The `eprintln` macro will panic if writing to stderr fails.
        let mut stderr = std::io::stderr().lock();
        #[rustfmt::skip]
        writeln!(stderr, "Updating to Minecraft version {}", version.as_str().yellow()).ok();

        // Get the latest build for the current version.
        writeln!(stderr, "Getting latest build for selected version...").ok();
        let build = version
            .builds()
            .with_context(|| "failed to get all builds")?
            .pop()
            .with_context(|| "no builds available")?;

        if build.experimental() && !self.allow_experimental {
            let message = format!(
                "Selected version is experimental. Use {} or set a stable version explicitly",
                "--allow-experimental".yellow()
            );

            let hint = get_latest_stable_version(&versions, version)
                .map(|v| format!("The latest stable version is '{}'", v.as_str()))
                .ok(); // Ignore the error if we are unable to get the latest stable version.

            return Err(crate::Error::new(message, None).with_hint(hint).into());
        }

        if !self.allow_downgrade {
            // TODO: Check the current version and ensure the target version is above it.
            // TODO: Use semver to implement Version comparison operators.
        }

        let directory = self.cwd.to_path_buf();
        writeln!(stderr, "Downloading server.jar file...").ok();
        update_server_jar(&directory, &build, self.timeout)?;

        // Update configuration file to reflect the new version.
        update_config_file(crate::config::Config::path(directory), version)
            .with_context(|| "failed to update version in Axiom.toml")?;

        #[rustfmt::skip]
        writeln!(stderr, "Server updated to Minecraft version {}", version.as_str().yellow()).ok();

        Ok(())
    }
}

fn get_latest_stable_version(
    supported_versions: &[paper::Version],
    selected: &paper::Version,
) -> Result<paper::Version, anyhow::Error> {
    let mut older_versions: Vec<&paper::Version> = supported_versions
        .iter()
        .take_while(|&v| v.as_str() != selected.as_str())
        .collect();

    while let Some(version) = older_versions.pop() {
        let build = version
            .builds()
            .with_context(|| "failed to get builds")?
            .pop()
            .with_context(|| "failed to get latest build")?;

        if build.stable() {
            return Ok(version.to_owned());
        }
    }

    Err(anyhow::anyhow!("failed to find the latest stable version"))
}

fn update_server_jar(
    base_directory: impl AsRef<std::path::Path>,
    build: &paper::Build,
    timeout: u64,
) -> Result<(), anyhow::Error> {
    let jars = base_directory.as_ref().join("jars");
    let paper_jar = jars.join(build.download_name());

    if !paper_jar.exists() {
        std::fs::create_dir_all(&jars).with_context(|| "Failed to create 'jars' directory")?;
        let data = build
            .download(std::time::Duration::from_secs(timeout))
            .with_context(|| "failed to download new server")?;

        std::fs::write(&paper_jar, &data).with_context(|| "failed to save new server")?;
    }

    let server_directory = base_directory.as_ref().join("server");

    // TODO: I don't think this should be an assertion since the user can call this subcommand
    // themselves. Perhaps suggest they run the build command first? should I just create the
    // directory for them? Why are you updating the server if you don't have a server yet? Is this
    // just for the convenience of not having to open the configuration file? Is that okay?
    debug_assert!(server_directory.exists());
    let server_jar = server_directory.join("server.jar");

    if let Err(err) = std::fs::remove_file(&server_jar) {
        match err.kind() {
            std::io::ErrorKind::NotFound => (),
            std::io::ErrorKind::IsADirectory => std::fs::remove_dir_all(&server_jar)?,
            _ => return Err(err.into()),
        }
    }

    symlink::symlink_file(&paper_jar, &server_jar)
        .with_context(|| "failed to link new server.jar")?;

    Ok(())
}

fn update_config_file(
    config_file: impl AsRef<std::path::Path>,
    version: &paper::Version,
) -> Result<(), anyhow::Error> {
    let mut doc = std::fs::read_to_string(&config_file)?.parse::<toml_edit::DocumentMut>()?;
    doc["server"]["version"] = toml_edit::value(version.as_str());
    std::fs::write(&config_file, doc.to_string())?;

    Ok(())
}
