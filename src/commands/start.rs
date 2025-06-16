use super::build::Build;
use crate::commands::Run;

#[derive(Debug, Clone, clap::Args)]
pub struct Start {
    #[clap(flatten)]
    pub cwd: crate::args::BaseDirectory,
}

impl Run for Start {
    fn run(&self) -> Result<(), anyhow::Error> {
        // Run the build command before starting the server.
        Build::run(&Build {
            cwd: self.cwd.clone(),
            accept_eula: false,
        })?;

        // Create a tmux session to run the server in.

        // Stream capture-output while scanning for the success or failure log messages.

        // TODO: I appear to be piggy backing off the other commands often... perhaps they should
        // be refactored so I can call the individual functions for finer control.

        todo!()
    }
}
