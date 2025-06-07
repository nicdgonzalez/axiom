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
            let hint = get_latest_stable_version(&versions, version)
                .map(|v| format!("The latest stable version is '{}'", v.as_str()))
                .unwrap_or_default();

            let message = format!(
                "Selected version is experimental. Use {} or set a stable version explicitly",
                "--allow-experimental".yellow()
            );

            return Err(crate::Error::new(message, None).with_hint(hint).into());
        }

        if !self.allow_downgrade {
            // TODO: Check the current version and ensure the target version is above it.
        }

        writeln!(stderr, "Downloading server.jar file...").ok();
        let directory = self.cwd.to_path_buf();
        let jars = directory.join("jars");
        let paper_jar = jars.join(build.download_name());

        if !paper_jar.exists() {
            std::fs::create_dir_all(&jars).with_context(|| "Failed to create 'jars' directory")?;
            let data = build
                .download(std::time::Duration::from_secs(self.timeout))
                .with_context(|| "failed to download new server")?;

            std::fs::write(&paper_jar, &data).with_context(|| "failed to save new server")?;
        }

        let server_directory = directory.join("server");
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

        // Update configuration file to use new version.
        let config_file = directory.join("Axiom.toml");
        let mut doc = std::fs::read_to_string(&config_file)?.parse::<toml_edit::DocumentMut>()?;
        doc["server"]["version"] = toml_edit::value(version.as_str());
        std::fs::write(&config_file, doc.to_string())?;

        #[rustfmt::skip]
        writeln!(stderr, "Server updated to Minecraft version {}", version.as_str().yellow()).ok();

        Ok(())
    }
}

fn get_latest_stable_version(
    supported_versions: &[paper::Version],
    selected: &paper::Version,
) -> anyhow::Result<paper::Version> {
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
