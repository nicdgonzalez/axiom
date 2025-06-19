use std::io::BufRead;

use anyhow::Context;
use colored::Colorize;

use crate::commands::Run;
use axiom_core::{Manifest, ManifestMut};

#[derive(Debug, clap::Args)]
pub struct Update {
    /// The version of Minecraft to use.
    pub(crate) version: Option<String>,

    /// Upgrade to the latest version, even if the latest version is not yet stable.
    #[arg(long, short = 'e')]
    pub(crate) allow_experimental: bool,

    /// Downgrade to an older version of Minecraft.
    #[arg(long, short = 'd')]
    pub(crate) allow_downgrade: bool,

    /// Seconds to wait before failing to download the new server JAR.
    #[arg(long, short = 't', default_value = "120")]
    pub(crate) timeout: u64,
}

impl Run for Update {
    fn run(&self, ctx: &crate::context::Context) -> Result<(), anyhow::Error> {
        // TODO: Create a static list of valid versions, and only fetch dynamically if the version
        // provided is not in the list. This way we can only call PaperMC for the latest build.
        let versions = paper::versions().with_context(|| "failed to get valid versions")?;

        let version = match &self.version {
            Some(version) => versions
                .iter()
                .find(|&entry| version == entry.as_str())
                .ok_or_else(|| anyhow::anyhow!("invalid version provided"))?,
            None => versions.last().with_context(|| "no versions available")?,
        };

        tracing::info!(
            "Checking latest build for Minecraft version {}...",
            version.as_str()
        );

        let build = version
            .builds()
            .with_context(|| "failed to get all builds")?
            .pop()
            .with_context(|| "no builds available")?;

        let manifest = ctx.manifest().with_context(|| "failed to get manifest")?;

        // You don't need to explicitly pass the experimental flag, if you are already on an
        // experimental build.
        let allow_experimental =
            if build.experimental() && (manifest.server.version == build.version()) {
                true
            } else {
                self.allow_experimental
            };

        if build.experimental() && !allow_experimental {
            let message = format!(
                "selected version is experimental. use {} or set a stable version explicitly",
                "--allow-experimental".yellow()
            );

            let hint = get_latest_stable_version(&versions, version)
                .map(|v| format!("The latest stable version is '{}'", v.as_str()))
                .ok(); // Returns `None` if we failed to get the latest stable version.

            return Err(crate::Error::new(message, None).with_hint(hint).into());
        }

        if !self.allow_downgrade {
            tracing::info!("Checking which version is currently installed...");
            if let Some(before) =
                installed_version(&ctx.cwd()).with_context(|| "failed to get installed version")?
            {
                ensure_no_downgrade(&before, version)?;
            }
        }

        let paper_jar = ctx.jars().join(build.download_name());

        if paper_jar.exists() {
            tracing::info!("Already using the latest build");
        } else {
            tracing::info!("Downloading the latest build...");

            let data = build
                .download(std::time::Duration::from_secs(self.timeout))
                .with_context(|| "failed to download new server")?;

            std::fs::create_dir_all(&ctx.jars())
                .with_context(|| "failed to create 'jars' directory")?;
            std::fs::write(&paper_jar, &data).with_context(|| "failed to save new server")?;
        }

        std::fs::create_dir_all(ctx.server())
            .with_context(|| "failed to create 'server' directory")?;

        let server_jar = ctx.server().join("server.jar");

        if let Err(err) = std::fs::remove_file(&server_jar) {
            match err.kind() {
                std::io::ErrorKind::NotFound => (), // No file to remove.
                std::io::ErrorKind::IsADirectory => std::fs::remove_dir_all(&server_jar)?,
                _ => return Err(err).with_context(|| "failed to remove existing server"),
            }
        }

        symlink::symlink_file(&paper_jar, &server_jar)
            .with_context(|| "failed to link new server.jar")?;

        // Update the configuration file to reflect the new version.
        let manifest_filepath = Manifest::filepath(ctx.cwd());

        let mut updated_manifest = ManifestMut::from_filepath(&manifest_filepath)?;
        updated_manifest.set_version(version.as_str());
        updated_manifest.set_build(i64::from(build.number()));

        std::fs::write(&manifest_filepath, updated_manifest.to_string())
            .with_context(|| "failed to set new version and build in the manifest")?;

        tracing::info!(
            "Server is running Minecraft version {} (#{})",
            version.as_str(),
            build.number()
        );

        Ok(())
    }
}

// Due to the long interval between Minecraft version releases, we typically see only one
// additional API call as the previous version usually stabilizes by the time a new one is
// released. However, this function can technically call the API multiple times if consecutive
// releases do not reach a stable status.
//
// TODO: It would be a good idea to limit the number of calls we can make or to cache information
// that will allow us to determine the latest stable version locally.
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

/// Checks which version of Minecraft the current `server.jar` is using.
///
/// This function reads the output after running the `server.jar` directly to ensure we get an
/// accurate version. This function is relatively slow.
fn installed_version<P>(cwd: P) -> Result<Option<paper::Version>, anyhow::Error>
where
    P: AsRef<std::path::Path>,
{
    let server = cwd.as_ref().join("server");
    let server_jar = server.join("server.jar");

    if !server_jar.exists() {
        return Ok(None);
    }

    let output = std::process::Command::new("java")
        .args([
            "-jar",
            server_jar
                .to_str()
                .with_context(|| "expected path to server.jar to be valid unicode")?,
            "--version",
        ])
        .current_dir(server)
        .output()
        .with_context(|| "failed to execute `java` command")?;

    tracing::debug!("Installed version output: {:?}", output.stdout);

    let version = output
        .stdout
        .lines()
        .last()
        .and_then(|line| line.ok()?.split("-").next().map(String::from))
        .with_context(|| "failed to get installed version")?;

    Ok(unsafe { Some(paper::Version::new(version)) })
}

fn ensure_no_downgrade(
    before: &paper::Version,
    after: &paper::Version,
) -> Result<(), anyhow::Error> {
    let before = semver::Version::parse(before.as_str())
        .expect("expected `before` to follow semantic versioning");
    let after = semver::Version::parse(after.as_str())
        .expect("expected `after` to follow semantic versioning");

    if let std::cmp::Ordering::Greater = before.cmp(&after) {
        let message = format!(
            "the selected version ({}) is older than the current version ({})",
            after, before
        );
        let hint = Some(format!(
            "try again with {} or use a different version",
            "--allow-downgrade".yellow()
        ));
        return Err(crate::Error::new(message, None).with_hint(hint).into());
    }

    Ok(())
}

/// Update the value of `version` in the server's configuration file.
fn update_version_in_config<P>(
    config_file: P,
    new_version: &paper::Version,
) -> Result<(), anyhow::Error>
where
    P: AsRef<std::path::Path>,
{
    let mut doc = std::fs::read_to_string(&config_file)?.parse::<toml_edit::DocumentMut>()?;
    doc["server"]["version"] = toml_edit::value(new_version.as_str());
    std::fs::write(&config_file, doc.to_string())
        .with_context(|| "failed to update the version in the configuration file")?;

    Ok(())
}
