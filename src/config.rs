use std::collections::BTreeMap;

#[allow(unused)]
#[derive(Debug, serde::Deserialize)]
pub struct Config {
    pub server: Server,
    pub properties: Option<Properties>,
}

#[allow(unused)]
#[derive(Debug, serde::Deserialize)]
pub struct Server {
    version: String,
}

#[allow(unused)]
#[derive(Debug, serde::Deserialize)]
pub struct Properties {
    #[serde(flatten)]
    items: BTreeMap<String, toml::Value>,
}

#[derive(Debug)]
pub enum ConfigError {
    NotFound {
        directory: std::path::PathBuf,
        source: Box<dyn std::error::Error + Send + Sync + 'static>,
    },
    ParseFailed {
        source: Box<dyn std::error::Error + Send + Sync + 'static>,
    },
    Io(std::io::Error),
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotFound {
                directory,
                source: _,
            } => write!(f, "could not find Axiom.toml in '{}'", directory.display()),
            Self::ParseFailed { source: _ } => write!(f, "failed to parse configuration file"),
            Self::Io(inner) => write!(f, "{inner}"),
        }
    }
}

impl std::error::Error for ConfigError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::NotFound {
                directory: _,
                source,
            } => Some(source.as_ref()),
            Self::ParseFailed { source } => Some(source.as_ref()),
            Self::Io(source) => Some(source),
        }
    }
}

impl ConfigError {
    /// Creates an error indicating the configuration file was not found.
    pub fn not_found(
        directory: impl AsRef<std::path::Path>,
        source: impl Into<Box<dyn std::error::Error + Send + Sync + 'static>>,
    ) -> Self {
        Self::NotFound {
            directory: directory.as_ref().to_path_buf(),
            source: source.into(),
        }
    }

    /// Creates an error indicating a failure to parse the configuration file.
    pub fn parse_failed(
        source: impl Into<Box<dyn std::error::Error + Send + Sync + 'static>>,
    ) -> Self {
        Self::ParseFailed {
            source: source.into(),
        }
    }
}

impl std::str::FromStr for Config {
    type Err = ConfigError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        toml::from_str(s).map_err(ConfigError::parse_failed)
    }
}

impl Config {
    const FILENAME: &'static str = "Axiom.toml";

    /// Read and parse the configuration file from the given directory.
    #[allow(unused)]
    pub fn from_directory<P>(directory: P) -> Result<Self, ConfigError>
    where
        P: AsRef<std::path::Path>,
    {
        let path = Self::path(directory);
        let contents = std::fs::read_to_string(&path).map_err(|err| match err.kind() {
            std::io::ErrorKind::NotFound => ConfigError::not_found(&path, err),
            _ => ConfigError::Io(err),
        })?;

        contents.parse::<Self>()
    }

    /// Creates the path to the configuration file from a given directory.
    ///
    /// This is a convenience function to join [`Self::FILENAME`] to the given directory.
    pub fn path<P>(directory: P) -> std::path::PathBuf
    where
        P: AsRef<std::path::Path>,
    {
        directory.as_ref().join(Self::FILENAME)
    }
}
