/// Represents an error that occurred while handling subcommands.
#[derive(Debug)]
pub struct Error {
    message: String,
    hint: Option<String>,
    source: Option<Box<dyn std::error::Error + Send + Sync + 'static>>,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        if let Some(source) = &self.source {
            Some(source.as_ref())
        } else {
            None
        }
    }
}

impl Error {
    /// Creates an error indicating a subcommand failed to execute.
    pub fn new(
        message: impl Into<String>,
        source: Option<Box<dyn std::error::Error + Send + Sync + 'static>>,
    ) -> Self {
        Self {
            message: message.into(),
            hint: None,
            source,
        }
    }

    /// Provide a message for the user that may help resolve the error, or erase a previous hint.
    pub fn with_hint(self, hint: Option<String>) -> Self {
        Self { hint, ..self }
    }

    /// Get the optional hint associated with the error.
    ///
    /// This method returns a user-friendly message intended to assist in resolving the error.
    ///
    /// # Examples
    ///
    /// ```
    /// # fn main() {
    /// let error = axiom::Error::new("configuration file not found".to_owned(), None)
    ///     .with_hint(Some("try running the `init` command first".to_owned()));
    ///
    /// if let Some(hint) = error.hint() {
    ///     eprintln!("Hint: {}", hint);
    /// }
    /// # }
    /// ```
    pub fn hint(&self) -> Option<&str> {
        self.hint.as_deref()
    }
}
