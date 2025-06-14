// Base directory flags to `#[clap(flatten)]` into your CLI.
#[derive(Debug, clap::Args)]
pub struct BaseDirectory {
    /// Change the current working directory.
    #[arg(long, short = 'C', global = true)]
    directory: Option<std::path::PathBuf>,
}

impl BaseDirectory {
    pub fn to_path_buf(&self) -> std::path::PathBuf {
        match self.directory {
            Some(ref path) => path.to_path_buf(),
            None => std::env::current_dir().expect("failed to get current directory"),
        }
    }
}
