//! This module defines the `Axiom.toml` file.

/// Contains all of the information about a package, as loaded from an `Axiom.toml` file.
///
/// # Examples
///
/// The easiest way to construct this type is via [`str::parse`]:
///
/// ```
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let input = r#"
///     [package]
///     name = "example"
///     version = "0.1.0"
///
///     [server]
///     version = "1.21.6"
///     build = 34
/// "#;
/// let manifest = input.parse::<axiom::Manifest>()
///     .expect("expected hard-coded input to be valid");
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Manifest {
    package: Package,
    server: Server,
    launcher: Option<Launcher>,
    properties: Option<Properties>,
}

impl std::str::FromStr for Manifest {
    type Err = ManifestError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        toml::from_str(s).map_err(|err| ManifestError::ParseFailed { source: err.into() })
    }
}

impl Manifest {
    /// A package manifest is typically loaded from an `Axiom.toml` file.
    pub const FILENAME: &'static str = "Axiom.toml";

    /// Create a new package manifest.
    pub fn new(
        package: Package,
        server: Server,
        launcher: Option<Launcher>,
        properties: Option<Properties>,
    ) -> Self {
        Self {
            package,
            server,
            launcher,
            properties,
        }
    }

    /// Get information related to the package, such as `name` and `version`.
    pub const fn package(&self) -> &Package {
        &self.package
    }

    /// Get information related to the Minecraft server being used.
    pub const fn server(&self) -> &Server {
        &self.server
    }

    /// Get information related to the generation of the `start.sh` script.
    pub const fn launcher(&self) -> Option<&Launcher> {
        self.launcher.as_ref()
    }

    /// Get the keys and values that will be written into the server's `server.properties` file.
    pub const fn properties(&self) -> Option<&Properties> {
        self.properties.as_ref()
    }

    /// Read and parse the manifest from the given base directory.
    ///
    /// This is a convenience function for joining `path` and [`Self::FILENAME`] then calling
    /// [`Self::from_file`].
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use axiom::Manifest;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let path = std::env::current_dir()?;
    /// assert!(path.join(Manifest::FILENAME).exists());
    /// let manifest = Manifest::from_directory(&path)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn from_directory<P>(path: P) -> Result<Self, ManifestError>
    where
        P: AsRef<std::path::Path>,
    {
        let file = path.as_ref().join(Self::FILENAME);
        Self::from_file(file)
    }

    /// Read and parse the manifest file from the given path.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use axiom::Manifest;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let path = std::env::current_dir()?.join(Manifest::FILENAME);
    /// assert!(path.exists());
    /// let manifest = Manifest::from_file(&path)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn from_file<P>(path: P) -> Result<Self, ManifestError>
    where
        P: AsRef<std::path::Path>,
    {
        use std::io::ErrorKind;
        let path = path.as_ref();

        let contents = std::fs::read_to_string(path).map_err(|err| match err.kind() {
            ErrorKind::NotFound => ManifestError::NotFound {
                path: path.to_owned(),
            },
            _ => ManifestError::ReadFailed { source: err.into() },
        })?;

        contents.parse()
    }
}

/// Describes an error that occurred while attempting to parse a manifest.
#[derive(Debug)]
pub enum ManifestError {
    /// Indicates a failure to locate the manifest file.
    NotFound {
        /// The path where the manifest was expected to be.
        path: std::path::PathBuf,
    },
    /// Indicates there was a problem reading the contents of the manifest file.
    ReadFailed {
        /// The underlying error that caused the failure.
        source: Box<dyn std::error::Error + Send + Sync + 'static>,
    },
    /// Indicates a failure to deserialize the manifest's contents.
    ParseFailed {
        /// The underlying error that caused the failure.
        source: Box<dyn std::error::Error + Send + Sync + 'static>,
    },
}

impl std::fmt::Display for ManifestError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotFound { path } => {
                write!(
                    f,
                    "could not find Axiom.toml in {}",
                    path.parent().unwrap().display()
                )
            }
            Self::ReadFailed { source: _ } => "failed to read manifest file".fmt(f),
            Self::ParseFailed { source: _ } => "failed to parse manifest".fmt(f),
        }
    }
}

impl std::error::Error for ManifestError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::NotFound { path: _ } => None,
            Self::ReadFailed { source } => Some(source.as_ref()),
            Self::ParseFailed { source } => Some(source.as_ref()),
        }
    }
}

/// Contains information related to the package.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Package {
    name: String,
    version: String,
}

impl Package {
    /// Construct a new "package" section for the manifest.
    ///
    /// # Examples
    ///
    /// ```
    /// use axiom::manifest::Package;
    ///
    /// # fn main() {
    /// let name = "example".to_owned();
    /// assert!(Package::valid_name(&name));
    /// let version = "0.1.0".to_owned();
    /// let package = Package::new(name, version);
    /// # }
    /// ```
    pub fn new(name: String, version: String) -> Self {
        Self { name, version }
    }

    /// Check if `name` works as a valid package name.
    ///
    /// The `name` will be used as window names in a tmux session. Package names should be
    /// unique as to not conflict with other running servers. It is recommended to store all
    /// packages in the same directory, and use the directory names as the package names.
    ///
    /// Package names should be alphanumeric and may contain dashes and underscores.
    /// Package names should not contain any colons (`:`) or periods (`.`).
    pub fn valid_name(name: &str) -> bool {
        name.chars()
            .all(|c| c.is_alphanumeric() || c == '_' || c == '-')
    }

    /// Get the name of the package.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get the package version.
    pub fn version(&self) -> &str {
        &self.version
    }
}

/// Contains information related to the Minecraft server being used.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Server {
    version: String,
    build: i64, // The `toml` crate uses `i64` for its integer value.
}

impl Server {
    /// Construct a new "server" section for the manifest.
    ///
    /// ```
    /// use axiom::manifest::Server;
    ///
    /// # fn main() {
    /// let version = "1.21.6".to_owned();
    /// let build = 34;
    /// let server = Server::new(version, build);
    /// # }
    /// ```
    pub fn new(version: String, build: i64) -> Self {
        Self { version, build }
    }

    /// Get the Minecraft server version.
    pub fn version(&self) -> &str {
        &self.version
    }

    /// Get the incremental build number for the server JAR release.
    pub fn build(&self) -> i64 {
        self.build
    }
}

/// Contains information related to the generation of the `start.sh` script.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Launcher {
    preset: Preset,
    memory: Option<String>,
    jvm_args: Option<Vec<String>>,
    game_args: Option<Vec<String>>,
}

impl Launcher {
    /// Construct a new "launcher" section for the manifest.
    ///
    /// # Examples
    ///
    /// ```
    /// use axiom::manifest::{Launcher, Preset};
    ///
    /// # fn main() {
    /// let preset = Preset::None;
    /// let memory = "4G".to_owned();
    /// let jvm_args = vec!["-XX:+UseG1GC".to_owned()];
    /// // let game_args = vec![];
    /// let launcher = Launcher::new(preset, Some(memory), Some(jvm_args), None);
    /// # }
    /// ```
    pub fn new(
        preset: Preset,
        memory: Option<String>,
        jvm_args: Option<Vec<String>>,
        game_args: Option<Vec<String>>,
    ) -> Self {
        Self {
            preset,
            memory,
            jvm_args,
            game_args,
        }
    }

    /// Get the preset configuration for the launcher.
    pub const fn preset(&self) -> &Preset {
        &self.preset
    }

    /// Specifies the maximum and initial memory allocation pool for the JVM (Java Virtual
    /// Machine).
    ///
    /// This value will be used for both the maximum and initial memory allocation pool.
    ///
    /// For details on valid values, see [this answer] from Stack Overflow.
    ///
    /// [this answer]: https://stackoverflow.com/a/32858015
    pub fn memory(&self) -> Option<&str> {
        self.memory.as_deref()
    }

    /// Get the command-line arguments that will be appended to the `java` command.
    ///
    /// The start command looks something like this:
    ///
    /// ```txt
    /// java -Xms[memory] -Xmx[memory] [preset] [jvm_args] -jar server.jar [game_args]
    /// ```
    pub fn jvm_args(&self) -> Option<&[String]> {
        self.jvm_args.as_deref()
    }

    /// Get the command-line arguments that will be appended to the end of the start command.
    ///
    /// The start command looks something like this:
    ///
    /// ```txt
    /// java -Xms[memory] -Xmx[memory] [preset] [jvm_args] -jar server.jar [game_args]
    /// ```
    pub fn game_args(&self) -> Option<&[String]> {
        self.game_args.as_deref()
    }
}

/// Preset command-line flags for the JVM (Java Virtual Machine) to enhance server performance.
///
/// Presets and flags were copied from [flags.sh].
///
/// [flags.sh]: https://flags.sh
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Preset {
    /// Skip adding any optimization flags.
    #[default]
    None,
    /// Use flags optimized for high performance.
    Aikars,
    /// Flags that work best with proxy software.
    Proxy,
}

impl Preset {
    /// Get a collection of command-line flags associated with this preset option.
    pub fn flags(&self) -> Vec<&'static str> {
        match self {
            Self::None => vec![],
            Self::Aikars => vec![
                "-XX:+UseG1GC",
                "-XX:+ParallelRefProcEnabled",
                "-XX:MaxGCPauseMillis=200",
                "-XX:+UnlockExperimentalVMOptions",
                "-XX:+DisableExplicitGC",
                "-XX:+AlwaysPreTouch",
                "-XX:G1HeapWastePercent=5",
                "-XX:G1MixedGCCountTarget=4",
                "-XX:InitiatingHeapOccupancyPercent=15",
                "-XX:G1MixedGCLiveThresholdPercent=90",
                "-XX:G1RSetUpdatingPauseTimePercent=5",
                "-XX:SurvivorRatio=32",
                "-XX:+PerfDisableSharedMem",
                "-XX:MaxTenuringThreshold=1",
                "-Dusing.aikars.flags=https://mcflags.emc.gs",
                "-Daikars.new.flags=true",
                "-XX:G1NewSizePercent=30",
                "-XX:G1MaxNewSizePercent=40",
                "-XX:G1HeapRegionSize=8M",
                "-XX:G1ReservePercent=20",
            ],
            Self::Proxy => vec![
                "-XX:+UseG1GC",
                "-XX:G1HeapRegionSize=4M",
                "-XX:+UnlockExperimentalVMOptions",
                "-XX:+ParallelRefProcEnabled",
                "-XX:+AlwaysPreTouch",
                "-XX:MaxInlineLevel=15",
            ],
        }
    }
}

/// Contains the keys and values that will be written into the server's `server.properties` file.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Properties {
    #[serde(flatten)]
    items: std::collections::BTreeMap<String, toml::Value>,
}

impl Properties {
    /// Construct a new "properties" section for the manifest.
    ///
    /// # Examples
    ///
    /// ```
    /// use axiom::manifest::Properties;
    /// use toml_edit::value;
    ///
    /// # fn main() {
    /// let mut items = std::collections::BTreeMap::<String, toml::Value>::new();
    /// items.insert("pvp".to_owned(), toml::Value::Boolean(true));
    /// items.insert("motd".to_owned(), "A Minecraft server".into());
    /// let properties = Properties::new(items);
    ///
    /// // NOTE: The entries are sorted in alphabetical order.
    /// let expected = "motd=A Minecraft server\npvp=true".to_owned();
    /// assert_eq!(properties.to_server_properties(), expected);
    /// # }
    /// ```
    pub fn new(items: std::collections::BTreeMap<String, toml::Value>) -> Self {
        Self { items }
    }

    /// Get the keys and values that will be copied into the server's `server.properties` file.
    pub fn items(&self) -> &std::collections::BTreeMap<String, toml::Value> {
        &self.items
    }

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
