use clap;

/// Argument parsing error type.
#[derive(Debug)]
pub enum ArgError {
    /// CLI error.
    ClapError(clap::Error),
    /// IO error.
    IoError(crate::io::IoError),
    /// Serde JSON error.
    SerdeJsonError(serde_json::Error),
    /// Serde YAML error.
    SerdeYamlError(serde_yaml::Error),
    /// TOML serialization error.
    TomlSerError(toml::ser::Error),
    /// TOML deserialization error.
    TomlDeError(toml::de::Error),
    /// StringOnly error.
    StringOnly(String),
}
impl std::fmt::Display for ArgError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ArgError::ClapError(error) => write!(f, "- CLI Error:\n{}", error),
            ArgError::IoError(error) => write!(f, "- IO Error:\n{}", error),
            ArgError::SerdeJsonError(error) => write!(f, "- JSON Serialization/Deserialization Error:\n{}", error),
            ArgError::SerdeYamlError(error) => write!(f, "- YAML Serialization/Deserialization Error:\n{}", error),
            ArgError::TomlSerError(error) => write!(f, "- TOML Serialization Error:\n{}", error),
            ArgError::TomlDeError(error) => write!(f, "- TOML Deserialization Error:\n{}", error),
            ArgError::StringOnly(error) => write!(f, "- {}", error),
        }
    }
}
impl From<clap::Error> for ArgError {
    fn from(error: clap::Error) -> Self {
        ArgError::ClapError(error)
    }
}
impl From<crate::io::IoError> for ArgError {
    fn from(error: crate::io::IoError) -> Self {
        ArgError::IoError(error)
    }
}
impl From<serde_json::Error> for ArgError {
    fn from(error: serde_json::Error) -> Self {
        ArgError::SerdeJsonError(error)
    }
}
impl From<serde_yaml::Error> for ArgError {
    fn from(error: serde_yaml::Error) -> Self {
        ArgError::SerdeYamlError(error)
    }
}
impl From<toml::ser::Error> for ArgError {
    fn from(error: toml::ser::Error) -> Self {
        ArgError::TomlSerError(error)
    }
}
impl From<toml::de::Error> for ArgError {
    fn from(error: toml::de::Error) -> Self {
        ArgError::TomlDeError(error)
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
