use std::{path, process};

use crate::errors::TmuxError;

pub struct Session {
    pub name: String,
}

impl Session {
    /// This function creates a new `Session` object. It does NOT map to
    /// `tmux new-session`. Use method `create` to build the session.
    pub fn new(name: &str) -> Self {
        Session {
            name: name.to_owned(),
        }
    }

    /// Check if the session is currently running.
    ///
    /// `tmux has-session`
    ///
    /// By default, tmux allows you to use names that are "close enough".
    /// e.g., if you have a session named "foobar" and are checking if
    /// a session named "foo" exists, tmux will assume you meant "foobar"
    /// and return `true`, even though "foo" does not actually exist.
    /// Because this mechanic is unintuitive and may lead to unexpected
    /// results, this method will only return `true` if the session name
    /// is an exact match.
    ///
    /// # Errors
    ///
    /// Returns an error if there is a problem executing the command.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// fn main() {
    ///     let session = tmux::Session::new("does_not_exist");
    ///     assert_eq!(session.exists().unwrap(), false);
    /// }
    /// ```
    pub fn exists(&self) -> Result<bool, TmuxError> {
        let result = process::Command::new("tmux")
            .args(["has-session", "-t", &format!("={}", self.name)])
            .stdout(process::Stdio::null())
            .stderr(process::Stdio::null())
            .status()
            .map_err(|_| TmuxError::FailedToRunCommand)?
            .success();

        Ok(result)
    }

    /// Start a new session.
    ///
    /// `tmux new-session`
    ///
    /// # Errors
    ///
    /// Returns an error if a session with the same name already exists,
    /// or if there is a problem executing the command.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// fn main() {
    ///     let session = tmux::Session::new("foo");
    ///     session.create(None).unwrap();
    ///     assert_eq!(session.exists().unwrap(), true);
    /// }
    /// ```
    pub fn create(&self, start_directory: Option<&path::PathBuf>) -> Result<(), TmuxError> {
        if self.exists()? {
            return Err(TmuxError::SessionAlreadyExists(self.name.clone()));
        }

        let mut command = process::Command::new("tmux");
        command
            .args(["new-session", "-d", "-s", &self.name])
            .stdout(process::Stdio::null())
            .stderr(process::Stdio::null());

        if let Some(directory) = start_directory {
            command.args([
                "-c",
                &directory
                    .to_str()
                    .expect("expected path to be valid unicode"),
            ]);
        }

        let status = command
            .status()
            .map_err(|_| TmuxError::FailedToRunCommand)?;

        assert!(status.success());
        Ok(())
    }

    /// Destroy the session, closing any windows linked to it.
    ///
    /// # Errors
    ///
    /// Throws an error if a session with the same name is not found, or there
    /// is a problem executing the tmux command in a subprocess.
    pub fn kill(&self) -> Result<(), TmuxError> {
        if !self.exists()? {
            return Err(TmuxError::SessionNotFound(self.name.clone()));
        }

        let status = process::Command::new("tmux")
            .args(["kill-session", "-t", &self.name])
            .stdout(process::Stdio::null())
            .stderr(process::Stdio::null())
            .status()
            .map_err(|_| TmuxError::FailedToRunCommand)?;

        assert!(status.success());
        Ok(())
    }
}
