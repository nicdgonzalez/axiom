//! Defines the command-line options that are exposed to the user.

#[derive(clap::Parser)]
#[command(version)]
pub struct AxiomCLI {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(clap::Subcommand)]
pub enum Command {
    /// Create a new Minecraft server
    Create {
        /// A unique name to identify the server
        name: String,
        /// The version of Minecraft to use
        version: Option<String>,
        /// Whether to prompt for confirmation before accepting the Minecraft EULA
        #[arg(short = 'y', long)]
        accept_eula: bool,
    },

    /// Remove an existing Minecraft server
    Delete {
        /// The unique name used to identify a server
        name: String,
        /// Automatically confirm the deletion without additional prompts
        #[arg(short = 'y', long)]
        assume_yes: bool,
    },

    /// Open a server's `server.properties` file using a terminal editor (e.g. Vim)
    Edit {
        /// The unique name used to identify a server
        name: String,
    },

    /// Display information about a server
    Info {
        /// The unique name used to identify a server
        name: String,
    },

    /// Display a list of all existing servers
    List,

    /// Open the specified server
    Start {
        /// The unique name used to identify a server
        name: String,
    },

    /// Check if a server is currently running
    Status {
        /// The unique name used to identify a server
        name: String,
    },

    /// Close the specified server
    Stop {
        /// The unique name used to identify a server
        name: String,
    },

    /// Change the version of Minecraft a server is using
    Update {
        /// The unique name used to identify a server
        name: String,
        /// The version of Minecraft to change to
        version: Option<String>,
    },
}
