mod commands;

use anyhow::Context;
use clap::Parser;
use colored::Colorize;

#[derive(clap::Parser)]
#[command(version)]
pub struct CLI {
    #[command(subcommand)]
    pub command: commands::Command,
}

/// The main entry point to the program.
fn main() {
    if let Err(why) = try_main() {
        #[rustfmt::skip]
        eprintln!("{}: {}: {}", "axiom".bold().cyan(), "error".bold().red(), why);
        std::process::exit(1);
    }
}

fn try_main() -> anyhow::Result<()> {
    axiom::init().with_context(|| "failed to initialize Axiom")?;
    let args = CLI::parse();
    commands::handle_command(&args.command)?;

    Ok(())
}
