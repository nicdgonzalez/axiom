use std::io::Write;

use anyhow::Context;
use colored::Colorize;

#[derive(Debug, Clone, clap::Args)]
pub struct Update {
    /// The version of Minecraft to use.
    pub(crate) version: Option<String>,

    /// An incremental counter unique to each build that helps track the progress of releases.
    pub(crate) build: Option<i64>,

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

impl crate::commands::Run for Update {
    fn run(&self, ctx: &mut crate::context::Context) -> Result<(), crate::error::Error> {
        tracing::info!("getting supported Minecraft versions from PaperMC");
        let versions = ctx
            .versions()
            .with_context(|| "failed to get supported Minecraft versions from PaperMC")?;

        // Check if the version provided is a valid version.
        let version = match self.version.as_ref() {
            Some(version) => versions
                .iter()
                .find(|&v| version == v.as_str())
                .with_context(|| "version not supported")?,
            None => versions
                .last()
                .with_context(|| "no supported versions available")?,
        };

        // Check if the build provided is a valid build.
        let build = match self.build.as_ref() {
            Some(build) => axiom::paper::Build::new(
                version.as_str().to_owned(),
                *build,
                // The `Default` channel indicates a stable build, which will bypass certain
                // validation checks. This is desired because in some cases we are the caller,
                // and we don't want to make multiple calls to the PaperMC API to verify
                // information that was already verified.
                axiom::paper::Channel::Default,
                format!("paper-{version}-{build}.jar", version = version.as_str()),
            ),
            None => version
                .builds()
                .with_context(|| "failed to get builds")?
                .pop()
                .with_context(|| "no builds available for selected version")?,
        };

        let package = ctx
            .package()
            .with_context(|| "failed to get package manifest")?;

        // If the user is already using an experimental build, bypass the safe upgrade check.
        let allow_experimental = if build.experimental()
            && (version.as_str() == package.manifest().server().version())
        {
            true
        } else {
            self.allow_experimental
        };

        if build.experimental() && !allow_experimental {
            let message = format!(
                "selected version is experimental. use {} or set a stable version explicitly",
                "--allow-experimental".yellow()
            );

            let err = crate::error::Error::new(anyhow::anyhow!(message));

            if let Ok(stable_version) = get_latest_stable_version(&versions, version) {
                let hint = format!("The latest stable version is '{}'", stable_version.as_str());
                return Err(err.with_hint(|| hint));
            }

            return Err(err);
        }

        // TODO: Clean up the following code.
        if !self.allow_downgrade {
            tracing::info!("Checking which version is currently installed");

            if let Ok(current_version) = package.server().build_info() {
                ensure_no_downgrade(
                    &axiom::paper::Version::new(current_version.version().to_owned()),
                    version,
                )?;
            }
        }

        let jars = ctx.jars().with_context(|| "failed to get server JARs")?;
        let paper_jar = jars.join(build.download_name());

        if paper_jar.exists() {
            tracing::info!("Already using the latest build");
        } else {
            tracing::info!("Downloading the latest build...");

            let data = build
                .download(std::time::Duration::from_secs(self.timeout))
                .with_context(|| "failed to download new server")?;

            std::fs::create_dir_all(jars).with_context(|| "failed to create 'jars' directory")?;
            std::fs::write(&paper_jar, &data).with_context(|| "failed to save new server")?;
        }

        assert!(&package.server().path().exists());
        let server_jar = package.server().server_jar();

        if let Err(err) = std::fs::remove_file(server_jar) {
            match err.kind() {
                std::io::ErrorKind::NotFound => (), // No file to remove.
                std::io::ErrorKind::IsADirectory => std::fs::remove_dir_all(server_jar)
                    .with_context(|| "failed to remove server.jar directory")?,
                _ => return Err(err).with_context(|| "failed to remove existing server")?,
            }
        }

        symlink::symlink_file(&paper_jar, server_jar)
            .with_context(|| "failed to link new server.jar")?;

        // Even though we already read the package manifest in `package`, we need the raw manifest
        // contents in order to edit the file while preserving the user's comments.
        let manifest_content = std::fs::read_to_string(package.manifest_path())
            .with_context(|| "failed to read manifest")?;
        let mut document = manifest_content
            .parse::<toml_edit::DocumentMut>()
            .with_context(|| "failed to parse manifest")?;

        document["server"]["version"] = toml_edit::value(version.as_str());
        document["server"]["build"] = toml_edit::value(build.number());

        std::fs::write(package.manifest_path(), document.to_string())
            .with_context(|| "failed to set new version and build in the manifest")?;

        // TODO: The package's manifest and our `context` are now out of sync. In this case it's
        // fine, because it's the end of the function, but I probably need to figure out a way to
        // make the edits go through the context to ensure they are always updated together.

        let mut stderr = std::io::stderr().lock();
        writeln!(
            stderr,
            "✨ server updated to Minecraft version {} (#{})",
            version.as_str(),
            build.number()
        )
        .ok();

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
    supported_versions: &[axiom::paper::Version],
    selected: &axiom::paper::Version,
) -> Result<axiom::paper::Version, anyhow::Error> {
    let mut older_versions: Vec<&axiom::paper::Version> = supported_versions
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

fn ensure_no_downgrade(
    before: &axiom::paper::Version,
    after: &axiom::paper::Version,
) -> Result<(), crate::error::Error> {
    let before = semver::Version::parse(before.as_str())
        .expect("expected `before` to follow semantic versioning");
    let after = semver::Version::parse(after.as_str())
        .expect("expected `after` to follow semantic versioning");

    if let std::cmp::Ordering::Greater = before.cmp(&after) {
        let message = format!(
            "the selected version ({}) is older than the current version ({})",
            after, before
        );

        let hint = format!(
            "try again with {} or use a different version",
            "--allow-downgrade".yellow()
        );

        return Err(crate::error::Error::new_with_hint(message, hint));
    }

    Ok(())
}
