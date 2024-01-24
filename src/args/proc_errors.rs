use clap;

/// Argument parsing error type.
#[derive(Debug)]
pub enum ArgError {
    /// CLI error.
    ClapError(clap::Error),
    /// IO error.
    IoError(std::io::Error),
    /// Serde error.
    SerdeError(serde_yaml::Error),
    /// StringOnly error.
    StringOnly(String),
}
impl std::fmt::Display for ArgError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ArgError::ClapError(error) => write!(f, "CLI Error:\n{}", error),
            ArgError::IoError(error) => write!(f, "IO Error:\n{}", error),
            ArgError::SerdeError(error) => write!(f, "Serde Error:\n{}", error),
            ArgError::StringOnly(error) => write!(f, "{}", error),
        }
    }
}
impl From<clap::Error> for ArgError {
    fn from(error: clap::Error) -> Self {
        ArgError::ClapError(error)
    }
}
impl From<std::io::Error> for ArgError {
    fn from(error: std::io::Error) -> Self {
        ArgError::IoError(error)
    }
}
impl From<serde_yaml::Error> for ArgError {
    fn from(error: serde_yaml::Error) -> Self {
        ArgError::SerdeError(error)
    }
}
impl From<String> for ArgError {
    fn from(error: String) -> Self {
        ArgError::StringOnly(error)
    }
}

/// Result type for the `args` crate.
pub type ProcResult<T> = std::result::Result<T, ArgError>;

/// Create a `ArgError::StringOnly` from a string.
pub fn err_str<T>(error_str: &str) -> ProcResult<T> {
    Err(ArgError::StringOnly(error_str.to_string()))
}
