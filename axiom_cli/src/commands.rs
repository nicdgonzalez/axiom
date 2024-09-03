//! Maps each of the command-line options to their handlers.

mod create;
mod delete;
mod edit;
mod list;
mod start;
mod update;

use crate::cli::Command;

pub fn handle_command(command: &Command) -> anyhow::Result<()> {
    match &command {
        Command::Create {
            name,
            version,
            accept_eula,
        } => create::handler(name, version, accept_eula),
        Command::Delete { name, assume_yes } => delete::handler(name, assume_yes),
        Command::Edit { name } => edit::handler(name),
        Command::Info { name: _ } => todo!(),
        Command::List => list::handler(),
        Command::Start { name } => start::handler(name),
        Command::Status { name: _ } => todo!(),
        Command::Stop { name: _ } => todo!(),
        Command::Update { name, version } => update::handler(name, version),
    }
}
