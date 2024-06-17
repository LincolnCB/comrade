mod proc_errors;

use clap::{
    Args,
    Parser,
    ValueEnum,
    Subcommand,
};
use strum::EnumIter;
use std::ffi::OsString;

pub use proc_errors::{
    ArgError,
    ProcResult,
    err_str,
};

/// Constrained Optimization for Magnetic Resonance Array Design tool.
#[derive(Debug)]
#[derive(Parser)]
#[clap(
    version,
    name = "comrade",
    author = "Lincoln Craven-Brightman",
    about = "Constrained Optimization for Magnetic Resonance Array Design tool",
    override_usage = 
"comrade <COMMAND> <STAGE> [OPTIONS]
    ⎡Config filepaths can be of the     ⎤
    ⎢following supported types:         ⎥
    ⎢   -YAML                           ⎥
    ⎢   -JSON                           ⎥
    ⎣   -TOML                           ⎦")]
pub struct ComradeCli {
    #[command(subcommand)]
    pub subcommand: SubCommand,
}

#[derive(Debug, Clone)]
#[derive(Subcommand)]
pub enum SubCommand {
    /// Run the comrade process.
    #[command(name = "run")]
    Run(RunArgs),
    /// Output example config file for a stage.
    #[command(name = "example-cfg")]
    Example(ExampleArgs),
}

/// Run command arguments.
#[derive(Debug, Clone)]
#[derive(Args)]
pub struct RunArgs {
    pub start_stage: RunStage,
    
    #[command(flatten)]
    pub shared_args: SharedArgs,
    
    /// End stage. If none, just run the start stage.
    #[arg(long = "to")]
    pub end_stage: Option<RunStage>,
    
    /// Layout config filepath.
    #[arg(long)]
    pub layout_cfg: Option<String>,
    
    /// Mesh config filepath.
    #[arg(long)]
    pub mesh_cfg: Option<String>,
    
    /// Simulation config filepath.
    #[arg(long)]
    pub sim_cfg: Option<String>,
    
    /// Matching config filepath.
    #[arg(long = "match_cfg")]
    pub matching_cfg: Option<String>,
}

#[derive(Debug, Clone)]
#[derive(Args)]
pub struct ExampleArgs {
    /// Stage to dump example for.
    pub stage: RunStage,

    /// Method name to display example cfg file for. If none, display all available methods for the stage.
    pub method: Option<String>,

    /// Output format.
    #[arg(short, long)]
    #[clap(default_value = "yaml")]
    pub format: Format,
}

/// Comrade stage to run or demonstrate
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

/// Output format for example config files.
#[derive(Debug, Clone)]
#[derive(ValueEnum, EnumIter)]
pub enum Format {
    Yaml,
    Json,
    Toml,
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
#[derive(Debug, Clone)]
#[derive(Args)]
pub struct SharedArgs {

    // #[arg(short, long = "larmor")]
    // /// REQUIRED. Larmor frequency in MHz.
    // pub larmor_mhz: f64,
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
