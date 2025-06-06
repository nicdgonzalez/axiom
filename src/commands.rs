mod status;

pub(crate) trait Run {
    fn run(&self) -> anyhow::Result<()>;
}

#[derive(Debug, clap::Subcommand)]
pub(crate) enum Subcommand {
    Status(status::Status),
}

pub(crate) fn handle_subcommand(subcommand: &Subcommand) -> anyhow::Result<()> {
    match subcommand {
        Subcommand::Status(handler) => handler.run(),
    }
}
