type StdError = dyn std::error::Error + Send + Sync + 'static;

/// Represents errors that can occur while requesting build information from PaperMC.
#[derive(Debug)]
pub enum RequestError {
    /// An error occurred while attempting to send a request to PaperMC.
    RequestFailed {
        /// The underlying error that caused the request to fail.
        source: Box<StdError>,
    },
    /// The PaperMC API took too long to return a response.
    ResponseTimedOut {
        /// The underlying error that caused the timeout.
        source: Box<StdError>,
    },
    /// The response received from PaperMC was not in the expected format.
    ParseResponseFailed {
        /// The underlying error that occurred while attempting to parse the response.
        source: Box<StdError>,
    },
}

impl std::fmt::Display for RequestError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::RequestFailed { source: _ } => write!(f, "failed to send request to PaperMC API"),
            Self::ResponseTimedOut { source: _ } => write!(f, "failed to get response body"),
            Self::ParseResponseFailed { source: _ } => write!(f, "failed to parse response body"),
        }
    }
}

impl std::error::Error for RequestError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::RequestFailed { source } => Some(source.as_ref()),
            Self::ResponseTimedOut { source } => Some(source.as_ref()),
            Self::ParseResponseFailed { source } => Some(source.as_ref()),
        }
    }
}

impl RequestError {
    /// Creates an error indicating that an API request has failed.
    pub fn request_failed(source: impl Into<Box<StdError>>) -> Self {
        Self::RequestFailed {
            source: source.into(),
        }
    }

    /// Creates an error indicating that the API response has timed out.
    pub fn response_timed_out(source: impl Into<Box<StdError>>) -> Self {
        Self::ResponseTimedOut {
            source: source.into(),
        }
    }

    /// Creates an error indicating a failure to parse the API response.
    pub fn parse_response_failed(source: impl Into<Box<StdError>>) -> Self {
        Self::ParseResponseFailed {
            source: source.into(),
        }
    }
}
