/// Layout process error type.
#[derive(Debug)]
pub enum LayoutError {
    /// IO error.
    IoError(std::io::Error),
    /// StringOnly error.
    StringOnly(String),
}
impl std::fmt::Display for LayoutError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LayoutError::IoError(error) => write!(f, "IO Error:\n{}", error),
            LayoutError::StringOnly(error) => write!(f, "{}", error),
        }
    }
}
impl From<std::io::Error> for LayoutError {
    fn from(error: std::io::Error) -> Self {
        LayoutError::IoError(error)
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
