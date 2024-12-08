use std::{error, fmt};

#[derive(Debug)]
pub enum TmuxError {
    FailedToRunCommand,
    SessionAlreadyExists(String),
    SessionNotFound(String),
}

impl fmt::Display for TmuxError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::FailedToRunCommand => {
                write!(f, "Failed to execute command 'tmux' in a subprocess")
            }
            Self::SessionAlreadyExists(ref name) => {
                writeln!(f, "A tmux session named '{name}' already exists")
            }
            Self::SessionNotFound(ref name) => {
                writeln!(f, "A tmux session named '{name}' not found")
            }
        }
    }
}

impl error::Error for TmuxError {}
