use anyhow::Context;
use colored::Colorize;
use std::io::Write;

use super::update::Update;
use crate::commands::Run;
use crate::config::Config;

#[derive(Debug, clap::Args)]
pub struct Build {
    #[clap(flatten)]
    pub(crate) cwd: crate::args::BaseDirectory,

    /// Accept the Minecraft EULA (End User License Agreement) without prompting for user input.
    #[arg(long, short = 'y')]
    pub(crate) accept_eula: bool,
}

impl Run for Build {
    fn run(&self) -> Result<(), anyhow::Error> {
        let directory = self.cwd.to_path_buf();
        let config = Config::from_path(Config::path(&directory))
            .with_context(|| "failed to load configuration")?;

        // Create the server directory, if not already exists.
        let server = directory.join("server");
        std::fs::create_dir_all(&server).with_context(|| "failed to create 'server' directory")?;

        let mut stderr = std::io::stderr().lock();

        // Run the update command to download the `server.jar` file.
        if let Err(err) = Update::run(&Update {
            version: Some(config.server.version),
            allow_experimental: true,
            allow_downgrade: true,
            timeout: 120,
            cwd: self.cwd.clone(),
        }) {
            return Err(
                crate::Error::new("failed to get server.jar".to_owned(), Some(err.into())).into(),
            );
        }

        let server_properties = server.join("server.properties");

        // Run the server once to generate the initial files.
        if !server_properties.exists() {
            writeln!(stderr, "Generating server files...").ok();
            let server_jar = server.join("server.jar");
            debug_assert!(server_jar.exists());

            let status = std::process::Command::new("java")
                .args([
                    "-jar",
                    server_jar
                        .to_str()
                        .expect("expected path to be valid unicode"),
                ])
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null())
                .status()
                .with_context(|| "failed to execute java command")?;

            tracing::debug!("Java exited with status code: {:?}", status.code());
        }

        if let Some(properties) = config.properties {
            // Overwrite `server.properties` with the properties in the config file. Any missing
            // keys are supposed to be generated automatically at startup by the server.
            std::fs::write(&server_properties, properties.to_server_properties())
                .with_context(|| "failed to update server.properties")?;
        }

        // Check if eula.txt contains `true`.
        let eula_txt = server.join("eula.txt");
        let eula_contents =
            std::fs::read_to_string(&eula_txt).with_context(|| "failed to read eula.txt")?;

        if !eula_contents.contains("eula=true") {
            if !self.accept_eula && !prompt_user_to_accept_eula() {
                // User was prompted to accept the EULA, but they chose not to.
                return Ok(());
            }

            #[rustfmt::skip]
            assert!(eula_contents.contains("eula=false"), "expected the auto-generated eula.txt");

            std::fs::write(&eula_txt, eula_contents.replace("eula=false", "eula=true"))
                .with_context(|| "failed to write to eula.txt")?;
        }

        // TODO: Plugins would be added/removed here, but first I would need to figure out a way to
        // identify and download plugins. For now, I can maybe add a path to an executable build
        // script that can run every time this command is ran.

        writeln!(stderr, "Server is ready!").ok();
        Ok(())
    }
}

/// Prompts the user using stdin to interactively accept the Minecraft EULA.
fn prompt_user_to_accept_eula() -> bool {
    println!(
        "{}: {}",
        "You must accept the Minecraft EULA before continuing".bold(),
        "https://aka.ms/MinecraftEULA".underline().cyan()
    );
    print!("{} {} (y/N): ", "*".cyan(), "Accept and continue?".bold());
    #[rustfmt::skip]
    std::io::stdout().flush().expect("failed to print full prompt");

    let mut input = String::new();
    std::io::stdin()
        .read_line(&mut input)
        .expect("failed to read from stdin");

    input.trim().to_lowercase() == "y"
}
