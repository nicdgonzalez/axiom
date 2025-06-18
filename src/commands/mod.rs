mod build;
mod start;
mod status;
mod status_ext;
mod update;

pub trait Run {
    fn run(&self) -> Result<(), anyhow::Error>;
}

#[derive(Debug, clap::Subcommand)]
pub enum Subcommand {
    /// Apply any changes to the configuration file onto the server.
    Build(build::Build),
    /// Run the server.
    Start(start::Start),
    /// Ping a Minecraft server to get basic information about it.
    Status(status::Status),
    /// Similar to status, but allows you to ping Minecraft servers without a config file.
    StatusExt(status_ext::StatusExt),
    /// Change which version of Minecraft a server is using.
    Update(update::Update),
}

pub fn handle_subcommand(subcommand: &Subcommand) -> anyhow::Result<()> {
    let result = match subcommand {
        Subcommand::Build(handler) => handler.run(),
        Subcommand::Start(handler) => handler.run(),
        Subcommand::Status(handler) => handler.run(),
        Subcommand::StatusExt(handler) => handler.run(),
        Subcommand::Update(handler) => handler.run(),
    };

    result
}
