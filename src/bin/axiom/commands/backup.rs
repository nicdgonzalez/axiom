pub mod new;

#[derive(clap::Args)]
pub struct Args {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(clap::Subcommand)]
pub enum Command {
    /// Compress a copy of a server's files into a tarball.
    New(new::Args),
}

pub fn handle_command(command: &Command) -> Result<(), anyhow::Error> {
    match &command {
        Command::New(args) => new::run(args),
    }
}
