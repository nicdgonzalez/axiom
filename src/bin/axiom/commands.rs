pub mod backup;

pub mod delete;
pub mod fork;
pub mod list;
pub mod new;
pub mod send_command;
pub mod start;
pub mod stop;
pub mod update;

#[derive(clap::Subcommand)]
pub enum Command {
    Backup(backup::Args),
    /// Create a new Minecraft server.
    New(new::Args),
    /// Permanently remove an existing Minecraft server.
    Delete(delete::Args),
    /// Create a new server from an existing server.
    Fork(fork::Args),
    /// Display all of the existing servers.
    List(list::Args),
    /// Send a command to the specified server.
    SendCommand(send_command::Args),
    /// Open a server, allowing players to connect to the world.
    Start(start::Args),
    /// Close a server, disconnecting all players.
    Stop(stop::Args),
    /// Change the version of Minecraft a server is using.
    Update(update::Args),
}

pub fn handle_command(command: &Command) -> Result<(), anyhow::Error> {
    match &command {
        Command::Backup(subcommand) => backup::handle_command(&subcommand.command),
        Command::New(args) => new::run(args),
        Command::Delete(args) => delete::run(args),
        Command::Fork(args) => fork::run(args),
        Command::List(args) => list::run(args),
        Command::SendCommand(args) => send_command::run(args),
        Command::Start(args) => start::run(args),
        Command::Stop(args) => stop::run(args),
        Command::Update(args) => update::run(args),
    };

    Ok(())
}
