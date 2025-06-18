use std::os::unix::fs::PermissionsExt;

use anyhow::Context;
use colored::Colorize;
use std::io::Write;

use super::update::Update;
use crate::commands::Run;
use axiom_core::Manifest;

#[derive(Debug, clap::Args)]
pub struct Build {
    /// Accept the Minecraft EULA (End User License Agreement) without prompting for user input.
    #[arg(long, short = 'y')]
    pub(crate) accept_eula: bool,
}

impl Run for Build {
    fn run(&self) -> Result<(), anyhow::Error> {
        let directory = std::env::current_dir().expect("failed to get current directory");
        let config = Manifest::from_filepath(Manifest::filepath(&directory))
            .with_context(|| "failed to load configuration")?;

        let server = directory.join("server");
        std::fs::create_dir_all(&server).with_context(|| "failed to create 'server' directory")?;

        let mut stderr = std::io::stderr().lock();

        tracing::info!("Running the update command to download the latest build");
        if let Err(err) = Update::run(&Update {
            version: Some(config.server.version),
            allow_experimental: true,
            allow_downgrade: true,
            timeout: 120,
        }) {
            return Err(
                crate::Error::new("failed to get server.jar".to_owned(), Some(err.into())).into(),
            );
        }

        let server_properties = server.join("server.properties");

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
                    "--initSettings",
                ])
                .current_dir(&server)
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
                // User was prompted to accept the EULA, but they said no.
                return Ok(());
            }

            #[rustfmt::skip]
            std::fs::write(&eula_txt, "eula=true")
                .with_context(|| "failed to write to eula.txt")?;
        }

        writeln!(stderr, "Generating start.sh script...").ok();
        let start_sh = server.join("start.sh");

        let java_args = config
            .launcher
            .as_ref()
            .and_then(|launcher| launcher.java_args.to_owned())
            .unwrap_or_else(|| vec![]);
        let game_args = config
            .launcher
            .as_ref()
            .and_then(|launcher| launcher.game_args.to_owned())
            .unwrap_or_else(|| vec![]);

        let contents = format!(
            "#!/usr/bin/bash\n\njava {} -jar server.jar {}",
            java_args.join(" "),
            game_args.join(" "),
        );
        std::fs::write(&start_sh, contents).with_context(|| "failed to write to start.sh")?;

        let metadata = start_sh
            .metadata()
            .with_context(|| "failed to get start.sh metadata")?;
        let permissions = metadata.permissions();
        let mode = permissions.mode() | 0o700; // Give the user permission to execute the file.
        std::fs::set_permissions(&start_sh, std::fs::Permissions::from_mode(mode))
            .with_context(|| "failed to make start.sh executable")?;

        if let Some(script) = config.server.post_build {
            writeln!(stderr, "Running post-build script...").ok();
            match std::process::Command::new(script)
                .current_dir(directory)
                .status()
                .with_context(|| "failed to execute custom post-build script")
                .and_then(|status| {
                    status
                        .code()
                        .with_context(|| "post-build script terminated due to signal")
                }) {
                #[rustfmt::skip]
                Ok(code) => writeln!(stderr, "Post-build script finished with exit code: {}", code).ok(),
                Err(err) => writeln!(stderr, "Failed to execute post-build script: {}", err).ok(),
            };
        }

        writeln!(stderr, "Ready!").ok();
        Ok(())
    }
}

/// Prompts the user to interactively accept the Minecraft EULA.
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
