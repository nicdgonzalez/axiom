//! # Axiom
//!
//! **Axiom** is a command-line tool for managing Minecraft servers.

#![warn(missing_docs)]

mod commands;

use clap::Parser;
use tracing_subscriber::prelude::*;

#[derive(Debug, clap::Parser)]
struct Args {
    #[command(subcommand)]
    command: commands::Command,

    /// Use verbose output (use `-vv` or `-vvv` for more verbose output).
    #[arg(long, short = 'v', action = clap::ArgAction::Count, global = true)]
    verbose: u8,
}

/// Represents the exit status of the application.
pub enum ExitCode {
    /// The program terminated without any errors.
    Success = 0,
    /// The program terminated due to an error.
    Failure = 1,
}

impl std::process::Termination for ExitCode {
    fn report(self) -> std::process::ExitCode {
        std::process::ExitCode::from(self as u8)
    }
}

/// The main entry point to the application.
pub fn run() -> anyhow::Result<ExitCode> {
    let args = Args::parse();
    let level_filter = {
        use tracing::level_filters::LevelFilter;
        match args.verbose {
            0 => LevelFilter::WARN,
            1 => LevelFilter::INFO,
            2 => LevelFilter::DEBUG,
            _ => LevelFilter::TRACE,
        }
    };

    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer().with_filter(level_filter))
        .init();

    commands::handle_command(&args.command).map(|()| ExitCode::Success)
}
