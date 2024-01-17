use clap::{
    Args,
    Parser,
    Subcommand,
};

use crate::layout;
use crate::matching;

/// Constrained Optimization for Magnetic Resonance Array Design tool.
#[derive(Debug, Parser)]
pub struct ComradeCli {
    #[clap(subcommand)]
    pub sub_command: RunStage,
}

/// Parser for the subcommands of the comrade binary using clap.
#[derive(Debug, Subcommand)]
pub enum RunStage {
    #[command(name = "layout")]
    /// Run the layout process only, outputting a layout file and optional mesh.
    Layout(LayoutCli),

    #[command(name = "match")]
    /// Run the matching process only, outputting a matching file.
    Matching(MatchingCli),

    #[command(name = "full")]
    /// Run the full process, outputting a layout file, optional mesh, and matching file.
    Full(FullCli),
}

/// Shared arguments, used in all commands. Compiled with clap.
#[derive(Debug, Args)]
pub struct SharedArgs {
    #[arg(short, long = "input")]
    /// Path to the input file (.ply/.stl for Layout surface input, .loops for Matching layout input).
    pub input_path: String,

    #[arg(short, long = "larmor")]
    /// Larmor frequency in MHz.
    pub larmor_mhz: f64,

    #[arg(short, long = "output")]
    /// Name of the output file (filetype extension will be added automatically).
    pub output_name: String,
}

/// Compiled arguments for the layout command. Compiled with clap.
#[derive(Debug, Args)]
pub struct LayoutCli{
    #[arg(value_enum)]
    style: layout::LayoutStyleCliEnum,

    #[command(flatten)]
    layout_args: layout::LayoutArgs,

    #[command(flatten)]
    shared_args: SharedArgs,
}

impl LayoutCli{
    /// Construct the layout style from the CLI enum.
    pub fn reconstruct(self) -> Result<(layout::LayoutStyle, SharedArgs), String>{
        Ok((self.style.construct(self.layout_args)?, self.shared_args))
    }
}

/// Compiled arguments for the matching command. Compiled with clap.
#[derive(Debug, Args)]
pub struct MatchingCli{
    #[command(flatten)]
    matching_args: matching::MatchingArgs,

    #[command(flatten)]
    shared_args: SharedArgs,
}

/// Compiled arguments for the full command. Compiled with clap.
#[derive(Debug, Args)]
pub struct FullCli{
    #[command(flatten)]
    layout_args: layout::LayoutArgs,

    #[command(flatten)]
    matching_args: matching::MatchingArgs,

    #[command(flatten)]
    shared_args: SharedArgs,
}
