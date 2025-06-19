mod build;
mod start;
mod status;
mod status_ext;
mod update;

use crate::context::Context;

pub trait Run {
    fn run(&self, ctx: &Context) -> Result<(), anyhow::Error>;
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
    let ctx = Context::new();

    let result = match subcommand {
        Subcommand::Build(handler) => handler.run(&ctx),
        Subcommand::Start(handler) => handler.run(&ctx),
        Subcommand::Status(handler) => handler.run(&ctx),
        Subcommand::StatusExt(handler) => handler.run(&ctx),
        Subcommand::Update(handler) => handler.run(&ctx),
    };

    result
}
