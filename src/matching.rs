mod networks;

/// Matching process error type.
#[derive(Debug)]
pub enum MatchingError {
    /// IO error.
    IoError(std::io::Error),
    /// StringOnly error.
    StringOnly(String),
}
impl std::fmt::Display for MatchingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MatchingError::IoError(error) => write!(f, "IO Error:\n{}", error),
            MatchingError::StringOnly(error) => write!(f, "{}", error),
        }
    }
}
impl From<std::io::Error> for MatchingError {
    fn from(error: std::io::Error) -> Self {
        MatchingError::IoError(error)
    }
}
impl From<String> for MatchingError {
    fn from(error: String) -> Self {
        MatchingError::StringOnly(error)
    }
}

/// Result type for the `matching` crate.
pub type Result<T> = std::result::Result<T, MatchingError>;

/// Create a `MatchingError::StringOnly` from a string.
pub fn err_str<T>(error_str: &str) -> Result<T> {
    Err(MatchingError::StringOnly(error_str.to_string()))
}
