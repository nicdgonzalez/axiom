// Base directory flags to `#[clap(flatten)]` into your CLI.
#[derive(Debug, Clone, clap::Args)]
pub struct BaseDirectory {
    /// Change the current working directory.
    #[arg(long, short = 'C', global = true)]
    directory: Option<std::path::PathBuf>,
}

impl BaseDirectory {
    pub fn new(path: std::path::PathBuf) -> Self {
        Self {
            directory: Some(path),
        }
    }

    pub fn to_path_buf(&self) -> std::path::PathBuf {
        match self.directory {
            Some(ref path) => path.to_path_buf(),
            None => std::env::current_dir().expect("failed to get current directory"),
        }
    }
}
