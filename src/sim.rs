
/// Simulation process error type.
#[derive(Debug)]
pub enum SimError {
    /// IO error.
    IoError(std::io::Error),
    /// StringOnly error.
    StringOnly(String),
}
impl std::fmt::Display for SimError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SimError::IoError(error) => write!(f, "IO Error:\n{}", error),
            SimError::StringOnly(error) => write!(f, "{}", error),
        }
    }
}
impl From<std::io::Error> for SimError {
    fn from(error: std::io::Error) -> Self {
        SimError::IoError(error)
    }
}
impl From<String> for SimError {
    fn from(error: String) -> Self {
        SimError::StringOnly(error)
    }
}

/// Result type for the `sim` crate.
pub type Result<T> = std::result::Result<T, SimError>;

/// Create a `SimError::StringOnly` from a string.
pub fn err_str<T>(error_str: &str) -> Result<T> {
    Err(SimError::StringOnly(error_str.to_string()))
}
