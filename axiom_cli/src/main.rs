mod cli;
mod commands;

use clap::Parser;

fn main() -> anyhow::Result<()> {
    if let Err(why) = simple_logger::SimpleLogger::new()
        .with_level(log::LevelFilter::Info)
        .env()
        .init()
    {
        eprintln!("Failed to initialize logger: {why}");
    }

    axiom::init().expect("Failed to initialize Axiom");

    let cli = cli::AxiomCLI::parse();
    commands::handle_command(&cli.command)
}
