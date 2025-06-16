use std::collections::BTreeMap;

#[derive(Debug, serde::Deserialize)]
pub struct Config {
    pub server: Server,
    pub launcher: Launcher,
    pub properties: Option<Properties>,
}

#[derive(Debug, serde::Deserialize)]
pub struct Server {
    pub version: String,
    /// Path to an executable script that can will be ran after a build.
    ///
    /// This can be used to install plugins, commit changes to version control, etc.
    pub post_build: Option<std::path::PathBuf>,
}

#[derive(Debug, serde::Deserialize)]
pub struct Launcher {
    pub args: Vec<String>,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct Properties {
    #[serde(flatten)]
    pub items: BTreeMap<String, toml::Value>,
}

impl Properties {
    #[allow(unused)]
    pub fn to_server_properties(&self) -> String {
        fn serialize_item(key: &str, value: &toml::Value, prefix: Option<String>) -> String {
            let prefix = prefix.unwrap_or_default();

            match value {
                toml::Value::String(v) => format!("{}{}={}", prefix, key, v.replace(":", "\\:")),
                toml::Value::Integer(v) => format!("{}{}={}", prefix, key, v),
                toml::Value::Float(v) => format!("{}{}={}", prefix, key, v),
                toml::Value::Boolean(v) => format!("{}{}={}", prefix, key, v),
                toml::Value::Datetime(_) => unimplemented!("datetime not supported"),
                toml::Value::Array(_) => unimplemented!("array not supported"),
                toml::Value::Table(v) => v
                    .iter()
                    .map(|(k, v)| serialize_item(k, v, Some(format!("{}{}.", prefix, key))))
                    .collect::<Vec<String>>()
                    .join("\n"),
            }
        }

        toml::Table::try_from(self)
            .expect("expected properties to be a valid TOML table")
            .iter()
            .map(|(k, v)| serialize_item(k, v, None))
            .collect::<Vec<String>>()
            .join("\n")
    }
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
    #[allow(unused)]
    pub fn not_found<P, E>(directory: P, source: E) -> Self
    where
        P: AsRef<std::path::Path>,
        E: Into<Box<dyn std::error::Error + Send + Sync + 'static>>,
    {
        Self::NotFound {
            directory: directory.as_ref().to_path_buf(),
            source: source.into(),
        }
    }

    /// Creates an error indicating a failure to parse the configuration file.
    pub fn parse_failed<E>(source: E) -> Self
    where
        E: Into<Box<dyn std::error::Error + Send + Sync + 'static>>,
    {
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

    /// Read and parse the configuration file from the given path.
    #[allow(unused)]
    pub fn from_path<P>(path: P) -> Result<Self, ConfigError>
    where
        P: AsRef<std::path::Path>,
    {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serialize() {
        let input = r#"
            [server]
            version = "1.21.5"

            [properties]
            pvp = true
            level-name = "world"
            rcon.password = ""
            rcon.port = 25575
        "#;

        let config = input.parse::<Config>().unwrap();
        // NOTE: BTreeMap sorts the keys alphabetically.
        let expected = "level-name=world\npvp=true\nrcon.password=\nrcon.port=25575";

        assert_eq!(&config.properties.unwrap().to_server_properties(), expected);
    }
}
