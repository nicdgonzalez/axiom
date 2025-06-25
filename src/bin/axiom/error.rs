/// A wrapper over [`anyhow::Error`] that provides an optional `hint` message.
#[derive(Debug)]
pub struct Error {
    inner: Box<dyn std::error::Error + Send + Sync + 'static>,
    hint: Option<String>,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.inner.fmt(f)
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.inner.source()
    }
}

impl From<::anyhow::Error> for Error {
    fn from(value: ::anyhow::Error) -> Self {
        Self {
            inner: value.into(),
            hint: None,
        }
    }
}

impl Error {
    pub fn new<E>(source: E) -> Self
    where
        E: Into<Box<dyn std::error::Error + Send + Sync + 'static>>,
    {
        Self {
            inner: source.into(),
            hint: None,
        }
    }

    #[allow(dead_code)]
    pub fn with_hint<H, F>(self, hint: F) -> Self
    where
        H: std::fmt::Display + Send + Sync + 'static,
        F: FnOnce() -> H,
    {
        Self {
            hint: Some(hint().to_string()),
            ..self
        }
    }

    #[allow(dead_code)]
    pub fn new_with_hint<H, E>(hint: H, source: E) -> Self
    where
        H: std::fmt::Display + Send + Sync + 'static,
        E: Into<Box<dyn std::error::Error + Send + Sync + 'static>>,
    {
        Self {
            inner: source.into(),
            hint: Some(hint.to_string()),
        }
    }

    pub fn hint(&self) -> Option<&str> {
        self.hint.as_deref()
    }
}

/// Like [`anyhow::bail!`], but wraps the error in our `Error` type.
///
/// # Examples
#[macro_export]
macro_rules! bail {
    ($message:literal $(,)?) => {
        return Err($crate::error::Error::new(::anyhow::anyhow!($message)));
    };
    ($error:expr $(,)?) => {
        return Err($crate::error::Error::new($error));
    };
    ($fmt:literal, $($arg:tt)*) => {
        return Err($crate::error::Error::new(::anyhow::anyhow!($fmt, $($arg)*)));
    };
}
