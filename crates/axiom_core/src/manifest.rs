//! Defines the valid sections, keys and values for the `Axiom.toml` file of a server.

use std::collections::BTreeMap;

/// The `Axiom.toml` file for each package is called its *manifest*.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Manifest {
    /// Defines metadata for setting up the Minecraft server.
    pub server: Server,
    /// Configuration options for generating the `start.sh` file.
    pub launcher: Option<Launcher>,
    /// Customize the `server.properties` file.
    pub properties: Option<Properties>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Server {
    /// The version of Minecraft the server is using.
    pub version: String,
    /// An incremental counter that indicates an official release for the PaperMC server JAR.
    pub build: u32,
    /// Path to an executable script that will be ran at the beginning of the build process.
    pub pre_build: Option<std::path::PathBuf>,
    /// Path to an executable script that will be ran at the end of the build process.
    pub post_build: Option<std::path::PathBuf>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Launcher {
    /// Controls the JVM's (Java Virtual Machine) heap memory allocation.
    pub memory: Memory,
    /// Arguments/flags to pass to the `java` command.
    pub java_args: Option<Vec<String>>,
    /// Arguments/flags to pass to the Minecraft server (e.g., "--nogui").
    pub game_args: Option<Vec<String>>,
}

// TODO: Validate memory string at deserialization time.
//
// The following Stack Overflow answer describes the valid suffixes:
// https://stackoverflow.com/a/32858015
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Memory(String);

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Properties {
    /// Represents the entries of the `server.properties` file.
    #[serde(flatten)]
    pub items: BTreeMap<String, toml::Value>,
}

impl Properties {
    /// Serialize the TOML properties into the format expected by the `server.properties` file.
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
pub enum ManifestError {
    /// Indicates a failure to locate the manifest file.
    NotFound {
        directory: std::path::PathBuf,
        source: Box<dyn std::error::Error + Send + Sync + 'static>,
    },
    /// Indicates the manifest file was found, but there was a problem reading its contents.
    ReadFailed {
        source: Box<dyn std::error::Error + Send + Sync + 'static>,
    },
    /// Indicates missing or invalid keys, values, sections, etc.
    ParseFailed {
        source: Box<dyn std::error::Error + Send + Sync + 'static>,
    },
}

impl std::fmt::Display for ManifestError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotFound {
                directory,
                source: _,
            } => write!(f, "could not find Axiom.toml in '{}'", directory.display()),
            Self::ReadFailed { source: _ } => write!(f, "failed to read file"),
            Self::ParseFailed { source: _ } => write!(f, "failed to parse configuration file"),
        }
    }
}

impl std::error::Error for ManifestError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::NotFound {
                directory: _,
                source,
            } => Some(source.as_ref()),
            Self::ReadFailed { source } => Some(source.as_ref()),
            Self::ParseFailed { source } => Some(source.as_ref()),
        }
    }
}

impl ManifestError {
    /// Creates an error indicating the configuration file was not found.
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

    /// Creates an error indicating a failure to read the contents of the configuration file.
    pub fn read_failed<E>(source: E) -> Self
    where
        E: Into<Box<dyn std::error::Error + Send + Sync + 'static>>,
    {
        Self::ReadFailed {
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

impl std::str::FromStr for Manifest {
    type Err = ManifestError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        toml::from_str(s).map_err(ManifestError::parse_failed)
    }
}

impl Manifest {
    const FILENAME: &'static str = "Axiom.toml";

    /// Read and parse the configuration file from the given base directory.
    ///
    /// This is a convenience function for getting the manifest path using [`Self::filepath`] and
    /// [`Self::from_filepath`].
    pub fn from_directory<P>(directory: P) -> Result<Self, ManifestError>
    where
        P: AsRef<std::path::Path>,
    {
        let filepath = Self::filepath(directory.as_ref());
        Self::from_filepath(filepath)
    }

    /// Read and parse the configuration file from the given filepath.
    pub fn from_filepath<P>(filepath: P) -> Result<Self, ManifestError>
    where
        P: AsRef<std::path::Path>,
    {
        let contents = std::fs::read_to_string(&filepath).map_err(|err| match err.kind() {
            std::io::ErrorKind::NotFound => ManifestError::not_found(&filepath, err),
            _ => ManifestError::read_failed(err),
        })?;

        contents.parse::<Self>()
    }

    /// Creates the path to the configuration file from a given directory.
    ///
    /// This is a convenience function to join [`Self::FILENAME`] to the given directory.
    pub fn filepath<P>(directory: P) -> std::path::PathBuf
    where
        P: AsRef<std::path::Path>,
    {
        directory.as_ref().join(Self::FILENAME)
    }
}

#[derive(Debug, Clone)]
pub struct ManifestMut {
    inner: toml_edit::DocumentMut,
}

impl std::str::FromStr for ManifestMut {
    type Err = ManifestError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self {
            inner: s.parse().map_err(ManifestError::parse_failed)?,
        })
    }
}

impl std::fmt::Display for ManifestMut {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.inner.to_string())
    }
}

impl ManifestMut {
    pub fn from_directory<P>(directory: P) -> Result<Self, ManifestError>
    where
        P: AsRef<std::path::Path>,
    {
        let filepath = Manifest::filepath(directory.as_ref());
        Self::from_filepath(filepath)
    }

    pub fn from_filepath<P>(filepath: P) -> Result<Self, ManifestError>
    where
        P: AsRef<std::path::Path>,
    {
        let contents = std::fs::read_to_string(&filepath).map_err(|err| match err.kind() {
            std::io::ErrorKind::NotFound => ManifestError::not_found(&filepath, err),
            _ => ManifestError::read_failed(err),
        })?;

        Ok(Self {
            inner: contents.parse().map_err(ManifestError::parse_failed)?,
        })
    }

    pub fn set_version(&mut self, version: &str) {
        self.inner["server"]["version"] = toml_edit::value(version);
    }

    pub fn set_build(&mut self, build: i64) {
        self.inner["server"]["build"] = toml_edit::value(build);
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

        let config = input.parse::<Manifest>().unwrap();
        // NOTE: BTreeMap sorts the keys alphabetically.
        let expected = "level-name=world\npvp=true\nrcon.password=\nrcon.port=25575";

        assert_eq!(&config.properties.unwrap().to_server_properties(), expected);
    }
}
