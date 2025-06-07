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
    pub fn new(
        message: String,
        source: Option<Box<dyn std::error::Error + Send + Sync + 'static>>,
    ) -> Self {
        Self {
            message,
            hint: None,
            source: if let Some(source) = source {
                Some(source)
            } else {
                None
            },
        }
    }

    pub fn with_hint(self, hint: String) -> Self {
        Self {
            hint: Some(hint),
            ..self
        }
    }

    pub fn hint(&self) -> Option<&str> {
        self.hint.as_ref().map(|hint| hint.as_str())
    }
}
