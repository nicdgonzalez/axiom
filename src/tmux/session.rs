//! A light wrapper over the session-related commands for tmux.

#[derive(Debug)]
pub enum SessionError {
    InvalidName,
    CommandFailure(std::io::Error),
    InvalidStartDirectory,
    SessionExists,
    SessionNotExists,
    Io(std::io::Error),
}

impl std::fmt::Display for SessionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidName => write!(
                f,
                "Session names may not be empty, or include periods or colons"
            ),
            Self::CommandFailure(io) => write!(f, "Failed to execute `tmux`: {io}"),
            Self::InvalidStartDirectory => {
                write!(f, "Expected start directory to be valid unicode")
            }
            Self::SessionExists => write!(f, "A session with the same name already exists"),
            Self::SessionNotExists => write!(f, "Session with the specified name does not exist"),
            Self::Io(inner) => write!(f, "{inner}"),
        }
    }
}

impl std::error::Error for SessionError {}

impl From<std::io::Error> for SessionError {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}

pub struct Session {
    pub name: String,
}

impl Session {
    /// Construct a new `Session`.
    ///
    /// Note: This function does not create a new tmux session. See [`Session::create`].
    ///
    /// # Errors
    ///
    /// This function returns an error if:
    ///
    /// - `name` is empty or includes a period or colon.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # fn main() {
    /// let session = tmux::Session::new("foo").unwrap();
    ///
    /// if !session.exists().expect("expected tmux to be installed") {
    ///     session.create(None).unwrap();
    /// }
    ///
    /// session.kill().ok();
    /// # }
    /// ```
    pub fn new(name: &str) -> Result<Self, SessionError> {
        if name.is_empty() || name.contains(".") || name.contains(":") {
            return Err(SessionError::InvalidName);
        }

        Ok(Self {
            name: name.to_owned(),
        })
    }

    /// Checks whether a session with the same name exists.
    ///
    /// This function corresponds to `tmux has-session`. This function returns `true` if a session
    /// with the exact same name exists.
    ///
    /// # Errors
    ///
    /// This function returns an error if:
    ///
    /// - There is a problem executing `tmux`.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # fn main() {
    /// # }
    /// ```
    pub fn exists(&self) -> Result<bool, SessionError> {
        let status = std::process::Command::new("tmux")
            .args(["has-session", &format!("-t={}", self.name)])
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status()
            .map_err(|err| SessionError::CommandFailure(err))?;

        Ok(status.success())
    }

    /// Start a new session.
    ///
    /// This function corresponds to `tmux new-session`.
    ///
    /// # Errors
    ///
    /// This function returns an error if:
    ///
    /// - There is a problem executing `tmux`.
    /// - `start_directory` is not a valid unicode.
    /// - A session with the same name already exists.
    ///
    /// Note: This function does not verify whether `start_directory` points to a valid directory.
    /// If the specified path does not exist, tmux will silently ignore it. Therefore, it is the
    /// caller's responsibility to ensure that the path exists before invoking this function.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # fn main() {
    /// # }
    /// ```
    pub fn create(&self, start_directory: Option<&std::path::Path>) -> Result<(), SessionError> {
        let mut args = vec!["new-session", "-d", "-s", &self.name];

        if let Some(path) = start_directory {
            args.extend([
                "-c",
                &path
                    .to_str()
                    .ok_or_else(|| SessionError::InvalidStartDirectory)?,
            ]);
        }

        let status = std::process::Command::new("tmux")
            .args(args)
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status()
            .map_err(|err| SessionError::CommandFailure(err))?;

        status
            .success()
            .then(|| Ok(()))
            .ok_or_else(|| SessionError::SessionExists)?
    }

    /// Destroy the session, closing any windows linked to it.
    pub fn kill(&self) -> Result<(), SessionError> {
        let status = std::process::Command::new("tmux")
            .args(["kill-session", &format!("-t={}", self.name)])
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status()
            .map_err(|err| SessionError::CommandFailure(err))?;

        status
            .success()
            .then(|| Ok(()))
            .ok_or_else(|| SessionError::SessionNotExists)?
    }
}
