mod status;

pub(crate) trait Run {
    fn run(&self) -> anyhow::Result<()>;
}

#[derive(Debug, clap::Subcommand)]
pub(crate) enum Command {
    /// Open the server, allowing players to connect to the world.
    Start,
    /// Close the server, disconnecting all players.
    Stop,
    /// Query information about a running server.
    Status(status::StatusCommand),
}

pub(crate) fn handle_command(command: &Command) -> anyhow::Result<()> {
    match command {
        Command::Status(handler) => handler.run(),
        _ => todo!(),
    }
}
