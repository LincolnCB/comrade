/// Mesh process error type.
#[derive(Debug)]
pub enum MeshError {
    /// IO error.
    IoError(crate::io::IoError),
    /// StringOnly error.
    StringOnly(String),
}
impl std::fmt::Display for MeshError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MeshError::IoError(error) => write!(f, "- IO Error:\n{}", error),
            MeshError::StringOnly(error) => write!(f, "- {}", error),
        }
    }
}
impl From<crate::io::IoError> for MeshError {
    fn from(error: crate::io::IoError) -> Self {
        MeshError::IoError(error)
    }
}
impl From<String> for MeshError {
    fn from(error: String) -> Self {
        MeshError::StringOnly(error)
    }
}

/// Result type for the `mesh` crate.
pub type ProcResult<T> = std::result::Result<T, MeshError>;

/// Create a `MeshError::StringOnly` from a string.
pub fn err_str<T>(error_str: &str) -> ProcResult<T> {
    Err(MeshError::StringOnly(error_str.to_string()))
}
