use std::io::{BufRead, Write};

use anyhow::Context;
use colored::Colorize;

use crate::commands::Run;

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

impl Run for Update {
    fn run(&self) -> Result<(), anyhow::Error> {
        let versions = paper::versions().with_context(|| "failed to get valid versions")?;

        let version = match &self.version {
            Some(version) => versions
                .iter()
                .find(|&entry| version == entry.as_str())
                .ok_or_else(|| anyhow::anyhow!("invalid version provided"))?,
            None => versions.last().with_context(|| "no versions available")?,
        };

        // The `eprintln` macro will panic if writing to stderr fails. Since printing our progress
        // isn't important to successfully run this command, writing to this lock will allow us to
        // continue processing the request, even if writing to stderr fails.
        let mut stderr = std::io::stderr().lock();
        #[rustfmt::skip]
        writeln!(stderr, "Updating to Minecraft version {}", version.as_str().yellow()).ok();

        writeln!(stderr, "Getting latest build for selected version...").ok();
        let build = version
            .builds()
            .with_context(|| "failed to get all builds")?
            .pop()
            .with_context(|| "no builds available")?;

        if build.experimental() && !self.allow_experimental {
            let message = format!(
                "selected version is experimental. use {} or set a stable version explicitly",
                "--allow-experimental".yellow()
            );

            let hint = get_latest_stable_version(&versions, version)
                .map(|v| format!("The latest stable version is '{}'", v.as_str()))
                .ok(); // Returns `None` if we failed to get the latest stable version.

            return Err(crate::Error::new(message, None).with_hint(hint).into());
        }

        let directory = self.cwd.to_path_buf();

        if !self.allow_downgrade {
            writeln!(stderr, "Checking which version is currently installed...").ok();
            if let Some(before) =
                installed_version(&directory).with_context(|| "failed to get installed version")?
            {
                ensure_no_downgrade(&before, version)?;
            }
        }

        writeln!(stderr, "Downloading server.jar file...").ok();
        update_server_jar(&directory, &build, self.timeout)?;

        // Update the configuration file to reflect the new version.
        update_version_in_config(crate::config::Config::path(directory), version)
            .with_context(|| "failed to update version in Axiom.toml")?;

        #[rustfmt::skip]
        writeln!(stderr, "Server updated to Minecraft version {}", version.as_str().yellow()).ok();

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
        let message = "the selected version is older than the current version.".to_owned();
        let hint = Some(format!(
            "try again with {} or use a different version",
            "--allow-downgrade".yellow()
        ));
        return Err(crate::Error::new(message, None).with_hint(hint).into());
    }

    Ok(())
}

/// Download the new server JAR and symlink it to `./server/server.jar`.
fn update_server_jar(
    base_directory: impl AsRef<std::path::Path>,
    build: &paper::Build,
    timeout: u64,
) -> Result<(), anyhow::Error> {
    let jars = base_directory.as_ref().join("jars");
    let paper_jar = jars.join(build.download_name());

    if !paper_jar.exists() {
        let data = build
            .download(std::time::Duration::from_secs(timeout))
            .with_context(|| "failed to download new server")?;

        std::fs::create_dir_all(&jars).with_context(|| "failed to create 'jars' directory")?;
        std::fs::write(&paper_jar, &data).with_context(|| "failed to save new server")?;
    }

    let server_directory = base_directory.as_ref().join("server");
    std::fs::create_dir_all(&server_directory)
        .with_context(|| "failed to create 'server' directory")?;

    let server_jar = server_directory.join("server.jar");

    if let Err(err) = std::fs::remove_file(&server_jar) {
        match err.kind() {
            std::io::ErrorKind::NotFound => (), // No file to remove.
            std::io::ErrorKind::IsADirectory => std::fs::remove_dir_all(&server_jar)?,
            _ => return Err(err).with_context(|| "failed to remove existing server"),
        }
    }

    symlink::symlink_file(&paper_jar, &server_jar)
        .with_context(|| "failed to link new server.jar")?;

    Ok(())
}

fn update_version_in_config<P>(
    config_file: P,
    version: &paper::Version,
) -> Result<(), anyhow::Error>
where
    P: AsRef<std::path::Path>,
{
    let mut doc = std::fs::read_to_string(&config_file)?.parse::<toml_edit::DocumentMut>()?;
    doc["server"]["version"] = toml_edit::value(version.as_str());
    std::fs::write(&config_file, doc.to_string())
        .with_context(|| "failed to update the version in the configuration file")?;

    Ok(())
}
