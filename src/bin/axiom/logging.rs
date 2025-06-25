#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, clap::Args)]
pub struct Verbosity {
    /// Use verbose output (or `-vv` and `-vvv` for more verbose output).
    #[arg(long, short = 'v', action = clap::ArgAction::Count, global = true)]
    verbose: u8,
}

impl Verbosity {
    pub(crate) fn level_filter(&self) -> tracing::level_filters::LevelFilter {
        use tracing::level_filters::LevelFilter;
        match self.verbose {
            0 => LevelFilter::WARN,
            1 => LevelFilter::INFO,
            2 => LevelFilter::DEBUG,
            _ => LevelFilter::TRACE,
        }
    }
}
