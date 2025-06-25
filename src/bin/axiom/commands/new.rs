use std::io::Write;

use anyhow::Context;

#[derive(clap::Args)]
pub struct New {
    /// Path for where to set up the new package.
    path: std::path::PathBuf,

    /// A name for the resulting package. Defaults to the directory name.
    #[clap(long)]
    name: Option<String>,

    /// Path to the existing Minecraft server subdirectory.
    #[clap(long)]
    server: Option<std::path::PathBuf>,

    /// Path to the existing Minecraft server JAR file.
    #[clap(long)]
    jar: Option<std::path::PathBuf>,

    /// Initialize a new git repository.
    #[clap(long)]
    git: bool,
}

impl crate::commands::Run for New {
    fn run(&self, ctx: &mut crate::context::Context) -> Result<(), crate::error::Error> {
        if self.path.exists() {
            crate::bail!("cannot run the `new` command on an existing directory");
        }

        std::fs::create_dir_all(&self.path)
            .with_context(|| "failed to create package directory")?;

        let server_path = self.path.join("server");
        if let Some(existing_server) = &self.server {
            // If the user has an existing server already, rename it.
            std::fs::rename(existing_server, &server_path)
                .with_context(|| "failed to move existing Minecraft server")?;
        } else {
            // Otherwise, create a new empty directory.
            std::fs::create_dir_all(&server_path)
                .with_context(|| "failed to create new 'server' directory")?;
        }

        let server_jar_path = server_path.join("server.jar");
        if let Some(existing_jar) = &self.jar {
            std::fs::rename(existing_jar, &server_jar_path)
                .with_context(|| "failed to move existing server JAR")?;
        };

        let server = axiom::package::Server::new(server_path, server_jar_path);

        // Get the version and build number to insert into the manifest.
        let (version, build) = if self.jar.is_some() {
            // Get the version from the existing server JAR.
            let build_info = server
                .build_info()
                .with_context(|| "failed to get build info from the existing server JAR")?;
            let version = build_info.version().to_owned();
            let build = build_info.build();

            (version, build)
        } else {
            // Fetch the latest build dynamically from PaperMC.
            // TODO: Add the `--allow-experimental` flag for this command too.
            let versions = ctx
                .versions()
                .with_context(|| "failed to get supported Minecraft versions from PaperMC")?
                .clone();

            let latest_build = versions
                .last()
                .with_context(|| "no supported Minecraft versions found")?
                .builds()
                .with_context(|| "failed to get builds for selected version")?
                .pop()
                .with_context(|| "no builds found")?;

            let version = latest_build.version().to_owned();
            let build = latest_build.number();

            (version, build)
        };

        // Create the `Axiom.toml` file.
        let mut manifest = toml_edit::DocumentMut::new();
        manifest["package"] = toml_edit::Item::Table(toml_edit::Table::new());
        manifest["package"]["name"] = {
            let name = match &self.name {
                Some(name) => name,
                // Default to the directory name.
                None => self
                    .path
                    .file_name()
                    .and_then(|name| name.to_str())
                    .with_context(|| "expected path to be valid unicode")?,
            };

            toml_edit::value(name)
        };
        manifest["package"]["version"] = toml_edit::value("0.1.0");
        manifest["server"] = toml_edit::Item::Table(toml_edit::Table::new());
        manifest["server"]["version"] = toml_edit::value(version);
        manifest["server"]["build"] = toml_edit::value(build);

        // If a `server.properties` file exists in `./server`, copy the properties into Axiom.toml.
        let server_properties = server.server_properties();
        if server_properties.exists() {
            manifest["properties"] = toml_edit::Item::Table(toml_edit::Table::new());

            tracing::warn!(
                "deserializing the `server.properties` file is currently unimplemented! \
                please copy over your server properties into Axiom.toml manually"
            );
        }

        let manifest_path = self.path.join("Axiom.toml");
        std::fs::write(&manifest_path, manifest.to_string())
            .with_context(|| "failed to create Axiom.toml file")?;

        if self.git {
            let gitignore = self.path.join(".gitignore");
            let ignore_items = ["/server"];

            std::fs::write(&gitignore, ignore_items.join("\n"))
                .with_context(|| "failed to create .gitignore file")?;

            if let Err(err) = initialize_git(&self.path) {
                tracing::warn!("failed to initialize git: {err}");
            }
        }

        let mut stderr = std::io::stderr().lock();
        // TODO: Provide better output:
        // (See start.rs and stop.rs for examples)
        writeln!(stderr, "🎉 package created successfully").ok();

        Ok(())
    }
}

fn initialize_git<P>(path: P) -> Result<(), anyhow::Error>
where
    P: AsRef<std::path::Path>,
{
    let status = std::process::Command::new("git")
        .current_dir(path.as_ref())
        .arg("init")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .with_context(|| "failed to execute command 'git'")?;

    if !status.success() {
        anyhow::bail!("failed to initialize git");
    }

    Ok(())
}
