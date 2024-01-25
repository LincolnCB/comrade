/// Simulation process error type.
#[derive(Debug)]
pub enum SimError {
    /// IO error.
    IoError(crate::io::IoError),
    /// StringOnly error.
    StringOnly(String),
}
impl std::fmt::Display for SimError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SimError::IoError(error) => write!(f, "- IO Error:\n{}", error),
            SimError::StringOnly(error) => write!(f, "- {}", error),
        }
    }
}
impl From<crate::io::IoError> for SimError {
    fn from(error: crate::io::IoError) -> Self {
        SimError::IoError(error)
    }
}
impl From<String> for SimError {
    fn from(error: String) -> Self {
        SimError::StringOnly(error)
    }
}

/// Result type for the `sim` crate.
pub type ProcResult<T> = std::result::Result<T, SimError>;

/// Create a `SimError::StringOnly` from a string.
pub fn err_str<T>(error_str: &str) -> ProcResult<T> {
    Err(SimError::StringOnly(error_str.to_string()))
}
