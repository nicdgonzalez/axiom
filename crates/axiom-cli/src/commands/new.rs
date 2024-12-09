//! Implementation for the `new` command.

use std::io::Write;

use anyhow::Context;
use colored::Colorize;

use super::delete::{run as delete, Args as DeleteArgs};
use super::update::{run as update, Args as UpdateArgs};

#[derive(clap::Args)]
pub struct Args {
    /// A unique name used to identify the server.
    pub name: String,
    /// The version of Minecraft to use.
    pub version: Option<String>,
    /// Automatically accept the Minecraft EULA without user input.
    #[arg(short = 'y', long)]
    pub accept_eula: bool,
    /// Allow the server to use an experimental build of Paper (if applicable).
    #[arg(long)]
    pub allow_experimental: bool,
}

/// Create a new Minecraft server.
pub fn run(args: &Args) -> Result<(), anyhow::Error> {
    let (name, server) = axiom::validate_server_not_exists(&args.name)?;
    std::fs::create_dir(&server).with_context(|| "failed to create server directory")?;

    // Use the `update` command to download the target server.jar file.
    if let Err(why) = update(&UpdateArgs {
        name: args.name.clone(),
        version: args.version.clone(),
        allow_experimental: args.allow_experimental,
        allow_downgrade: false,
    }) {
        delete(&DeleteArgs {
            name: args.name.clone(),
            assume_yes: true,
        })?;
        return Err(why).with_context(|| "failed to download server.jar");
    }

    eprintln!("Running server to generate initial files...");
    if let Err(why) = generate_initial_files(&server) {
        delete(&DeleteArgs {
            name: args.name.clone(),
            assume_yes: true,
        })?;
        return Err(why).with_context(|| "failed to start server");
    }

    if !args.accept_eula && !prompt_user_to_accept_eula() {
        delete(&DeleteArgs {
            name: args.name.clone(),
            assume_yes: true,
        })?;
        return Ok(());
    }

    let eula_txt = server.join("eula.txt");
    accept_eula(&eula_txt)?;

    println!("{}", format!("Server '{name}' is ready!").green());
    Ok(())
}

fn generate_initial_files(server: &std::path::PathBuf) -> Result<(), anyhow::Error> {
    std::process::Command::new("java")
        .args(["-jar", "server.jar"])
        .current_dir(&server)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()?
        .wait()?;

    Ok(())
}

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

fn accept_eula(eula_txt: &std::path::PathBuf) -> Result<(), anyhow::Error> {
    let contents = std::fs::read_to_string(&eula_txt)
        .with_context(|| "failed to read eula.txt")?
        .replace("eula=false", "eula=true");
    std::fs::write(&eula_txt, &contents).with_context(|| "failed to write to eula.txt")?;

    Ok(())
}
