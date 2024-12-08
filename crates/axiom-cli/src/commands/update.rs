//! Implementation for the `update` command.

use std::{cmp::Ordering, fs, path};

use anyhow::{anyhow, Context};
use axiom::ChannelKind;
use colored::Colorize;
use semver::Version;

#[derive(clap::Args)]
pub struct Args {
    /// The unique name used to identify the server.
    pub name: String,
    /// The version of Minecraft to use.
    pub version: Option<String>,
    /// Allow the server to use an experimental build of Paper (if applicable).
    ///
    /// This only applies if you are creating a new server when a new version
    /// of Minecraft is released and Paper hasn't yet released a stable build
    /// for the new server yet.
    #[arg(long)]
    pub allow_experimental: bool,
    /// Allow the server to downgrade to an older version of Minecraft.
    #[arg(long)]
    pub allow_downgrade: bool,
}

/// Change the version of Minecraft a server is running.
pub fn run(args: &Args) -> Result<(), anyhow::Error> {
    let (name, server) = axiom::validate_server_exists(&args.name)?;

    let versions = axiom::get_paper_server_versions()?;
    let mut version = match &args.version {
        Some(version) => versions
            .iter()
            .find(|entry| *entry == version)
            .ok_or_else(|| anyhow!("invalid version"))?,
        None => versions.last().with_context(|| "no versions available")?,
    };
    assert!(Version::parse(version).is_ok());

    eprintln!(
        "Attempting to update to Minecraft version {}",
        version.cyan()
    );

    let mut build = axiom::get_paper_build_latest(&version)?;

    if !args.allow_experimental && build.channel == ChannelKind::Experimental {
        eprintln!(
            "Version {} is marked as experimental, but '{}' was not set",
            version.cyan(),
            "--allow-experimental".yellow()
        );
        let mut versions_lower: Vec<&String> =
            versions.iter().take_while(|v| *v != version).collect();

        while build.channel == ChannelKind::Experimental {
            version = versions_lower
                .pop()
                .with_context(|| "no lower versions available")?;
            eprintln!("Attempting to use version {} instead.", version.cyan());
            build = axiom::get_paper_build_latest(&version)?;
        }
    }

    let server_jar = server.join("server.jar");

    if !args.allow_downgrade {
        let version_installed = axiom::get_version_installed(&server_jar);
        #[rustfmt::skip]
        validate_no_downgrade(version_installed, &version)
            .with_context(|| {
                format!("flag '{}' not set", "--allow-downgrade".yellow())
            })?;
    }

    let paper_jar = get_paper_jar(&build)?;

    if server_jar.exists() {
        fs::remove_file(&server_jar).with_context(|| "failed to remove server.jar")?;
    }

    symlink::symlink_file(&paper_jar, &server_jar)
        .with_context(|| "failed to link target JAR to server.jar")?;

    #[rustfmt::skip]
    println!("{}", format!("Server {name} is now running Minecraft version: {}", version.cyan()).green());
    Ok(())
}

fn validate_no_downgrade(
    version_installed: Option<String>,
    version_target: &str,
) -> Result<(), anyhow::Error> {
    if let Some(existing) = version_installed {
        let version_old = Version::parse(&existing).with_context(|| {
            "expected link to be a file with format: paper-{version}-{build}.jar"
        })?;
        // Should have already been validated prior to calling this function.
        let version_new = Version::parse(version_target).unwrap();

        if let Ordering::Greater = version_old.cmp(&version_new) {
            return Err(anyhow!(
                "attempting to downgrade from {} to {}",
                existing.cyan(),
                version_target.cyan(),
            ));
        }
    }

    Ok(())
}

fn get_paper_jar(build: &axiom::Build) -> Result<path::PathBuf, anyhow::Error> {
    let build = axiom::get_paper_build_latest(&build.version)?;
    let paper_jar_path = axiom::get_jars_path()?.join(&build.filename);

    if !paper_jar_path.exists() {
        let paper_jar = axiom::get_paper_server_jar(&build)?;
        fs::write(&paper_jar_path, &paper_jar.data)
            .with_context(|| "failed to save downloaded server.jar")?;
    } else {
        eprintln!(
            "{}",
            "Target version installed previously. Skipping download".yellow()
        );
    }

    Ok(paper_jar_path)
}
