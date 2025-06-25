mod build;
mod list;
mod new;
mod start;
mod status;
mod status_ext;
mod stop;
mod update;

use crate::context::Context;
use crate::error::Error;

pub(crate) const TMUX_SERVER_NAME: &str = "axiom";
pub(crate) const TMUX_SESSION_NAME: &str = "servers";

pub(crate) trait Run {
    /// Execute the subcommand.
    fn run(&self, ctx: &mut Context) -> Result<(), Error>;
}

#[derive(clap::Subcommand)]
pub(crate) enum Subcommand {
    /// Apply any changes to the server.
    Build(build::Build),

    /// Display which Minecraft servers are currently active.
    List(list::List),

    /// Create a new package.
    New(new::New),

    /// Run the server, allowing players to connect to the world.
    Start(start::Start),

    /// Ping the Minecraft server to get basic information about it.
    Status(status::Status),

    /// Like `status`, but can ping external servers using only a hostname.
    StatusExt(status_ext::StatusExt),

    /// Close the server, disconnecting all players.
    Stop(stop::Stop),

    /// Use a different Minecraft version.
    Update(update::Update),
}

impl Subcommand {
    pub(crate) fn run(&self) -> Result<(), Error> {
        let mut ctx = Context::default();
        self.handler().run(&mut ctx)
    }

    pub(crate) fn handler(&self) -> &dyn Run {
        match self {
            Self::Build(handler) => handler,
            Self::List(handler) => handler,
            Self::New(handler) => handler,
            Self::Start(handler) => handler,
            Self::Status(handler) => handler,
            Self::StatusExt(handler) => handler,
            Self::Stop(handler) => handler,
            Self::Update(handler) => handler,
        }
    }
}
