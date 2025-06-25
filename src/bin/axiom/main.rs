mod commands;
mod context;
mod error;
mod logging;

use std::io::Write;

use clap::Parser;
use colored::Colorize;
use tracing_subscriber::prelude::*;

use crate::logging::Verbosity;

#[derive(clap::Parser)]
struct Args {
    #[command(subcommand)]
    command: commands::Subcommand,

    #[clap(flatten)]
    verbose: Verbosity,
}

/// Describes the result of the process after it has terminated.
#[derive(Debug, Clone, Copy)]
enum ExitCode {
    /// The program terminated without any errors.
    Success,
    /// The program terminated due to an unrecoverable error.
    Failure,
}

impl std::process::Termination for ExitCode {
    fn report(self) -> std::process::ExitCode {
        std::process::ExitCode::from(self as u8)
    }
}

/// The main entry point to the application.
fn main() -> ExitCode {
    try_main().unwrap_or_else(|err| {
        let mut stderr = std::io::stderr().lock();
        writeln!(stderr, "{}", "an error occurred".bold().red()).ok();

        let mut current_error: Option<&dyn std::error::Error> = Some(&err);

        while let Some(cause) = current_error {
            writeln!(stderr, "  {}: {}", "Cause".bold(), cause).ok();
            current_error = cause.source();
        }

        if let Some(hint) = err.hint() {
            writeln!(stderr, "  {}: {}", "Hint".bold().green(), hint).ok();
        }

        ExitCode::Failure
    })
}

fn try_main() -> Result<ExitCode, crate::error::Error> {
    let args = Args::parse();
    let level_filter = args.verbose.level_filter();

    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer().with_filter(level_filter))
        .init();

    args.command.run().map(|()| ExitCode::Success)
}
