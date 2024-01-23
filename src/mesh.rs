
/// Mesh process error type.
#[derive(Debug)]
pub enum MeshError {
    /// IO error.
    IoError(std::io::Error),
    /// StringOnly error.
    StringOnly(String),
}
impl std::fmt::Display for MeshError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MeshError::IoError(error) => write!(f, "IO Error:\n{}", error),
            MeshError::StringOnly(error) => write!(f, "{}", error),
        }
    }
}
impl From<std::io::Error> for MeshError {
    fn from(error: std::io::Error) -> Self {
        MeshError::IoError(error)
    }
}
impl From<String> for MeshError {
    fn from(error: String) -> Self {
        MeshError::StringOnly(error)
    }
}

/// Result type for the `mesh` crate.
pub type Result<T> = std::result::Result<T, MeshError>;

/// Create a `MeshError::StringOnly` from a string.
pub fn err_str<T>(error_str: &str) -> Result<T> {
    Err(MeshError::StringOnly(error_str.to_string()))
}
