use thiserror::Error;

/// Result type for state resolution.
pub type Result<T> = std::result::Result<T, Error>;

/// Represents the various errors that arise when resolving state.
#[derive(Error, Debug)]
pub enum Error {
    /// An error occurred while doing IO.
    #[error(transparent)]
    IoError(#[from] std::io::Error),

    /// An error occurred while converting to `String`.
    #[error(transparent)]
    Utf8Error(#[from] std::string::FromUtf8Error),

    #[error("{0}")]
    Unmatched(String),

    #[error("{0}")]
    Custom(Box<dyn std::error::Error>),
}

impl Error {
    pub fn custom<E: std::error::Error + 'static>(e: E) -> Self {
        Self::Custom(Box::new(e))
    }
}
