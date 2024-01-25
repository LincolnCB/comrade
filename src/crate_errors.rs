use crate::{
    args,
    layout,
    mesh,
    sim,
    matching,
};

/// Error-type enum for the `comrade` crate.
/// Can handle errors from the `clap` crate and the `stl_io` crate.
/// Will handle other errors in the future.
#[derive(Debug)]
pub enum ComradeError {
    ArgError(args::ArgError),
    LayoutError(layout::LayoutError),
    MeshError(mesh::MeshError),
    SimError(sim::SimError),
    MatchingError(matching::MatchingError),
    StringOnly(String),
}
impl std::fmt::Display for ComradeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ComradeError::ArgError(error) => write!(f, "! ARGUMENT ERROR:\n{}", error),
            ComradeError::LayoutError(error) => write!(f, "! LAYOUT ERROR:\n{}", error),
            ComradeError::MeshError(error) => write!(f, "! MESHING ERROR:\n{}", error),
            ComradeError::SimError(error) => write!(f, "! SIMULATION ERROR:\n{}", error),
            ComradeError::MatchingError(error) => write!(f, "! MATCHING ERROR:\n{}", error),
            ComradeError::StringOnly(error) => write!(f, "! COMRADE ERROR:\n- {}", error),
        }
    }
}
impl From<String> for ComradeError {
    fn from(error: String) -> Self {
        ComradeError::StringOnly(error)
    }
}
impl From<args::ArgError> for ComradeError {
    fn from(error: args::ArgError) -> Self {
        ComradeError::ArgError(error)
    }
}
impl From<layout::LayoutError> for ComradeError {
    fn from(error: layout::LayoutError) -> Self {
        ComradeError::LayoutError(error)
    }
}
impl From<mesh::MeshError> for ComradeError {
    fn from(error: mesh::MeshError) -> Self {
        ComradeError::MeshError(error)
    }
}
impl From<sim::SimError> for ComradeError {
    fn from(error: sim::SimError) -> Self {
        ComradeError::SimError(error)
    }
}
impl From<matching::MatchingError> for ComradeError {
    fn from(error: matching::MatchingError) -> Self {
        ComradeError::MatchingError(error)
    }
}

/// Result type for the `comrade` crate.
pub type ComradeResult<T> = std::result::Result<T, ComradeError>;

/// Create a `ComradeResult` with an `Err` from a string.
/// Shorthand to avoid writing `Err(crate::ComradeError::StringOnly(error_str))`.
pub fn err_str<T>(error_str: &str) -> ComradeResult<T> {
    Err(ComradeError::StringOnly(error_str.to_string()))
}
