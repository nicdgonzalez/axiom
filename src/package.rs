//! This module implements functionality for reading and interacting with an Axiom package.

use std::io::BufRead;

/// Represents the manifest and all of the files associated with it.
#[derive(Debug, Clone)]
pub struct Package {
    path: std::path::PathBuf,
    manifest: crate::Manifest,
    manifest_path: std::path::PathBuf,
    server: Server,
}

impl Package {
    /// Construct a new package.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// ```
    pub fn new(path: std::path::PathBuf, manifest: crate::Manifest) -> Self {
        let manifest_path = path.join(crate::Manifest::FILENAME);
        let server_path = path.join("server");
        let server_jar_path = server_path.join("server.jar");
        let server = Server::new(server_path, server_jar_path);

        Self {
            path,
            manifest,
            manifest_path,
            server,
        }
    }

    /// Get the path to the root directory of the package.
    pub fn path(&self) -> &std::path::Path {
        &self.path
    }

    /// Get the path to the package manifest.
    pub fn manifest_path(&self) -> &std::path::Path {
        &self.manifest_path
    }

    /// Get a reference to the contents of the manifest file.
    pub fn manifest(&self) -> &crate::Manifest {
        &self.manifest
    }

    /// Get the name of the package.
    pub fn name(&self) -> &str {
        self.manifest.package().name()
    }

    /// Get the version of the package.
    pub fn version(&self) -> &str {
        self.manifest.package().version()
    }

    /// Get a reference to the contents of the directory containing the Minecraft server.
    pub fn server(&self) -> &Server {
        &self.server
    }
}

/// Represents the `server` directory of a package.
///
/// The `server` directory contains the Minecraft server and all of its configuration files.
#[derive(Debug, Clone)]
pub struct Server {
    path: std::path::PathBuf,
    server_jar: std::path::PathBuf,
    server_properties: std::path::PathBuf,
    eula_txt: std::path::PathBuf,
    start_sh: std::path::PathBuf,
    logs: std::path::PathBuf,
}

impl Server {
    /// Represents the directory containing the Minecraft server.
    pub fn new(path: std::path::PathBuf, server_jar: std::path::PathBuf) -> Self {
        let server_properties = path.join("server.properties");
        let eula_txt = path.join("eula.txt");
        let start_sh = path.join("start.sh");
        let logs = path.join("logs");

        Self {
            path,
            server_jar,
            server_properties,
            eula_txt,
            start_sh,
            logs,
        }
    }

    /// Get the path to the directory containing the Minecraft server and its configuration files.
    pub fn path(&self) -> &std::path::Path {
        &self.path
    }

    /// Get the path to the server's `server.jar` file.
    ///
    /// The `server.jar` file is a symbolic link to the downloaded server JAR from PaperMC.
    pub fn server_jar(&self) -> &std::path::Path {
        &self.server_jar
    }

    /// Get the path to the server's `server.properties` file.
    ///
    /// The `server.properties` file contains configuration options for the Minecraft server.
    pub fn server_properties(&self) -> &std::path::Path {
        &self.server_properties
    }

    /// Get the path to the server's `eula.txt` file.
    ///
    /// The contents of the `eula.txt` file indicates to the server whether the server owner has
    /// accepted the Minecraft EULA (End User License Agreement) or not. This file should contain
    /// either `eula=true` or `eula=false`.
    pub fn eula_txt(&self) -> &std::path::Path {
        &self.eula_txt
    }

    /// Get the path to the server's `start.sh` file.
    ///
    /// The `start.sh` file contains the command to run the server JAR.
    pub fn start_sh(&self) -> &std::path::Path {
        &self.start_sh
    }

    /// Get the path to the server's `logs` directory.
    pub fn logs(&self) -> &std::path::Path {
        &self.logs
    }

    /// Get the version of Minecraft the current `server.jar` is running.
    ///
    /// This function queries the `server.jar` directly to ensure we get accurate version
    /// information. Because we are creating a subprocess and running the JAR directly, this
    /// operation is relatively slow (and even slower if it's the first time running the JAR).
    ///
    /// # Panics
    ///
    /// This function panics if the path to the `server.jar` file contains invalid unicode.
    ///
    /// # Errors
    ///
    /// This function returns an error if:
    ///
    /// - The `server.jar` file does not exist.
    /// - The command to run the `server.jar` file fails to execute.
    /// - We fail to parse the required information from the command's output.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # Ok(())
    /// # }
    /// ```
    pub fn build_info(&self) -> Result<ServerBuildInfo, ServerBuildInfoError> {
        if !self.server_jar.exists() {
            return Err(ServerBuildInfoError::ServerJarNotFound {
                path: self.server_jar.to_owned(),
            });
        }

        let command = "java";
        let output = std::process::Command::new(command)
            .current_dir(&self.path)
            .args([
                "-jar",
                self.server_jar
                    .to_str()
                    .expect("expected path to server.jar to be valid unicode"),
                "--version",
            ])
            .output()
            .map_err(|err| ServerBuildInfoError::CommandFailed {
                command: command.to_owned(),
                source: err.into(),
            })?;

        let current_version = output
            .stdout
            .lines()
            .last()
            .and_then(|line| {
                let line = line.ok()?;
                let mut parts = line.split("-"); // [version]-[build]-[commit_hash]
                let version = parts.next()?.to_owned();
                let build = parts.next()?.parse().ok()?;
                let commit_hash = parts.next()?.to_owned();
                Some(ServerBuildInfo::new(version, build, commit_hash))
            })
            .ok_or_else(|| ServerBuildInfoError::ParseFailed)?;

        Ok(current_version)
    }

    /// Check whether the Minecraft EULA (End User License Agreement) has been accepted.
    ///
    /// This function reads the server's `eula.txt` file and searches for the string `eula=true`.
    ///
    /// # Errors
    ///
    /// This function returns an error if:
    ///
    /// - There is a problem reading the `eula.txt` file.
    pub fn has_accepted_eula(&self) -> Result<bool, std::io::Error> {
        let contents = std::fs::read_to_string(self.eula_txt())?;
        Ok(contents.contains("eula=true"))
    }
}

/// Describes basic version information about a PaperMC server JAR file.
pub struct ServerBuildInfo(String, i64, String);

impl ServerBuildInfo {
    /// Describes a server JAR build.
    pub fn new(version: String, build: i64, commit_hash: String) -> Self {
        Self(version, build, commit_hash)
    }

    /// Represents a server JAR's build information after parsing the output from running the JAR
    /// with `--version`.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let server_jar_path = std::env::current_dir()?.join("server.jar");
    /// assert!(server_jar_path.exists());
    /// assert!(axiom::package::ServerBuildInfo::from_server_jar(&server_jar_path).is_ok());
    /// # Ok(())
    /// # }
    /// ```
    pub fn from_server_jar<P>(path: P) -> Result<Self, ServerBuildInfoError>
    where
        P: AsRef<std::path::Path>,
    {
        let path = path.as_ref();

        if !path.exists() {
            return Err(ServerBuildInfoError::ServerJarNotFound {
                path: path.to_owned(),
            });
        }

        let command = "java";
        let output = std::process::Command::new(command)
            .current_dir(path.parent().unwrap())
            .args([
                "-jar",
                path.to_str()
                    .expect("expected path to server.jar to be valid unicode"),
                "--version",
            ])
            .output()
            .map_err(|err| ServerBuildInfoError::CommandFailed {
                command: command.to_owned(),
                source: err.into(),
            })?;

        let current_version = output
            .stdout
            .lines()
            .last()
            .and_then(|line| {
                let line = line.ok()?;
                let mut parts = line.split("-"); // [version]-[build]-[commit_hash]
                let version = parts.next()?.to_owned();
                let build = parts.next()?.parse().ok()?;
                let commit_hash = parts.next()?.to_owned();
                Some(Self::new(version, build, commit_hash))
            })
            .ok_or_else(|| ServerBuildInfoError::ParseFailed)?;

        Ok(current_version)
    }

    /// Get the version of Minecraft the server JAR contains.
    pub fn version(&self) -> &str {
        &self.0
    }

    /// Get the build number of the current server JAR.
    ///
    /// After every release from PaperMC for a given Minecraft version, an incremental counter is
    /// increased. This number serves as an identifier for the server JAR.
    pub fn build(&self) -> i64 {
        self.1
    }

    /// Get the git commit hash for the current build.
    pub fn commit_hash(&self) -> &str {
        &self.2
    }
}

/// Describes an error that occurred while getting a server JAR's build information.
#[derive(Debug)]
pub enum ServerBuildInfoError {
    /// Indicates the server JAR file did not exist at the expected path.
    ServerJarNotFound {
        /// The path to the target server JAR file.
        path: std::path::PathBuf,
    },
    /// Indicates a failure to run the command needed to get the server JAR's information.
    CommandFailed {
        /// The name of the command we attempted to run.
        command: String,
        /// The underlying error that caused the command failure.
        source: Box<dyn std::error::Error + Send + Sync + 'static>,
    },
    /// Indicates a failure to extract the required information from the command's output.
    ParseFailed,
}

impl std::fmt::Display for ServerBuildInfoError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ServerJarNotFound { path } => write!(
                f,
                "could not find 'server.jar' in {}",
                path.parent().unwrap().display()
            ),
            Self::CommandFailed { command, source: _ } => {
                write!(f, "failed to execute command '{}'", command)
            }
            Self::ParseFailed => "failed to parse current version from command output".fmt(f),
        }
    }
}

impl std::error::Error for ServerBuildInfoError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::ServerJarNotFound { path: _ } => None,
            Self::CommandFailed { command: _, source } => Some(source.as_ref()),
            Self::ParseFailed => None,
        }
    }
}
