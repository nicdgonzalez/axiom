mod status;
mod status_ext;
mod update;

pub trait Run {
    fn run(&self) -> Result<(), anyhow::Error>;
}

#[derive(Debug, clap::Subcommand)]
pub enum Subcommand {
    /// Ping a Minecraft server to get basic information about it.
    Status(status::Status),
    /// Like `status`, but allows you to ping Minecraft servers using only a hostname.
    StatusExt(status_ext::StatusExt),
    /// Change which version of Minecraft a server is using.
    Update(update::Update),
}

pub fn handle_subcommand(subcommand: &Subcommand) -> anyhow::Result<()> {
    match subcommand {
        Subcommand::Status(handler) => handler.run(),
        Subcommand::StatusExt(handler) => handler.run(),
        Subcommand::Update(handler) => handler.run(),
    }
}
