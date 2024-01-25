/// Layout process error type.
#[derive(Debug)]
pub enum LayoutError {
    /// IO error.
    IoError(crate::io::IoError),
    /// Serde JSON error.
    SerdeJsonError(serde_json::Error),
    /// StringOnly error.
    StringOnly(String),
}
impl std::fmt::Display for LayoutError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LayoutError::IoError(error) => write!(f, "- IO Error:\n{}", error),
            LayoutError::SerdeJsonError(error) => write!(f, "- JSON Serialization/Deserialization Error:\n{}", error),
            LayoutError::StringOnly(error) => write!(f, "- {}", error),
        }
    }
}
impl From<crate::io::IoError> for LayoutError {
    fn from(error: crate::io::IoError) -> Self {
        LayoutError::IoError(error)
    }
}
impl From<serde_json::Error> for LayoutError {
    fn from(error: serde_json::Error) -> Self {
        LayoutError::SerdeJsonError(error)
    }
}
impl From<String> for LayoutError {
    fn from(error: String) -> Self {
        LayoutError::StringOnly(error)
    }
}

/// Result type for the `layout` crate.
pub type ProcResult<T> = std::result::Result<T, LayoutError>;

/// Create a `LayoutError::StringOnly` from a string.
pub fn err_str<T>(error_str: &str) -> ProcResult<T> {
    Err(LayoutError::StringOnly(error_str.to_string()))
}
