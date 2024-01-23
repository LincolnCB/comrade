use clap::{
    Args,
    Parser,
    ValueEnum,
};
use strum::EnumIter;
use std::ffi::OsString;

/// Argument parsing error type.
#[derive(Debug)]
pub enum ArgError {
    ClapError(clap::Error),
    IoError(std::io::Error),
    StringOnly(String),
}
impl std::fmt::Display for ArgError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ArgError::ClapError(error) => write!(f, "CLI Error:\n{}", error),
            ArgError::IoError(error) => write!(f, "IO Error:\n{}", error),
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

/// Result type for the `args` crate.
pub type Result<T> = std::result::Result<T, ArgError>;

/// Create a `ArgError::StringOnly` from a string.
pub fn err_str<T>(error_str: &str) -> Result<T> {
    Err(ArgError::StringOnly(error_str.to_string()))
}

/// Constrained Optimization for Magnetic Resonance Array Design tool.
#[derive(Debug, Parser)]
#[clap(
    name = "comrade",
    version = "0.1.0",
    author = "Lincoln Craven-Brightman",
    about = "Constrained Optimization for Magnetic Resonance Array Design tool",
    override_usage = "comrade <START_STAGE> [OPTIONS]")]
pub struct ComradeCli {

    pub start_stage: RunStage,

    #[command(flatten)]
    pub shared_args: SharedArgs,

    /// End stage. If none, just run the start stage.
    #[arg(long = "to")]
    pub end_stage: Option<RunStage>,

    /// Layout config filepath (YAML).
    #[arg(long)]
    pub layout_cfg: Option<String>,

    /// Mesh config filepath (YAML).
    #[arg(long)]
    pub mesh_cfg: Option<String>,

    /// Simulation config filepath (YAML).
    #[arg(long)]
    pub sim_cfg: Option<String>,

    /// Matching config filepath (YAML).
    #[arg(long = "match_cfg")]
    pub matching_cfg: Option<String>,
}

/// Run stage. Used as start and optional end of comrade process.
#[derive(Debug, Clone)]
#[derive(ValueEnum, EnumIter)]
pub enum RunStage {
    Layout,
    Mesh,
    Sim,
    Match,
}

impl RunStage {
    /// Get the stage number
    pub fn stage_num(&self) -> u32 {
        match self {
            RunStage::Layout => 1,
            RunStage::Mesh => 2,
            RunStage::Sim => 3,
            RunStage::Match => 4,
        }
    }
}

impl std::fmt::Display for RunStage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RunStage::Layout => write!(f, "layout"),
            RunStage::Mesh => write!(f, "mesh"),
            RunStage::Sim => write!(f, "sim"),
            RunStage::Match => write!(f, "match"),
        }
    }
}

/// Shared arguments, used in all commands. Compiled with clap.
#[derive(Debug, Args)]
pub struct SharedArgs {
    #[arg(short, long = "larmor")]
    /// REQUIRED. Larmor frequency in MHz.
    pub larmor_mhz: f64,
}

/// Re-export clap CLI parse method.
pub fn parse_cli_args() -> ComradeCli {
    ComradeCli::parse()
}

// Re-export clap CLI parse_from method
pub fn parse_cli_from<I, T>(itr: I) -> ComradeCli 
where
    I: IntoIterator<Item = T>,
    T: Into<OsString> + Clone,
{
    ComradeCli::parse_from(itr)
}
