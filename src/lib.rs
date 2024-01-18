pub mod layout;
pub mod matching;
pub mod args;

use std::io;
use clap;

/// Targets struct.
/// This struct contains the layout and matching targets to do.
pub struct Targets{
    pub layout_target: Option<layout::LayoutStyle>,
    pub matching_target: Option<u32>, // TODO THIS IS A DUMMY
    pub shared_args: args::SharedArgs,
}

/// Result type for the `comrade` crate.
type Result<T> = std::result::Result<T, ComradeError>;

/// Error-type enum for the `comrade` crate.
/// Can handle errors from the `clap` crate and the `stl_io` crate.
/// Will handle other errors in the future.
#[derive(Debug)]
pub enum ComradeError {
    IOError(io::Error),
    ClapError(clap::Error),
    Default(String),
}
impl From<io::Error> for ComradeError {
    fn from(error: io::Error) -> Self {
        ComradeError::IOError(error)
    }
}
impl From<clap::Error> for ComradeError {
    fn from(error: clap::Error) -> Self {
        ComradeError::ClapError(error)
    }
}
impl From<String> for ComradeError {
    fn from(error: String) -> Self {
        ComradeError::Default(error)
    }
}

/// Create a `Result` with an `Err` from a string.
/// Shorthand to avoid writing `Err(crate::ComradeError::Default(error_str))`.
pub fn err_string<T>(error_str: String) -> crate::Result<T> {
    Err(ComradeError::Default(error_str))
}
    

/// [Stage 1.] TODO UNFINISHED FUNCTION
/// Parse the command line arguments for the comrade binary.
/// Uses the `clap` crate.
/// Returns a `Result` with a `comrade::RunArgs` struct or an `Err`.
pub fn handle_cli_args(cli_args : args::ComradeCli) -> crate::Result<Targets>{

    // 1.1 Handle the different subcommands
    match cli_args.sub_command {
        args::RunStage::Layout(layout_cli) => { // Layout command
            println!("Layout only");
            println!("{:?}", layout_cli);

            let (layout_style, shared_args) = layout_cli.reconstruct()?;
            Ok(Targets{
                layout_target: Some(layout_style),
                matching_target: None,
                shared_args,
            })
            
        },
        args::RunStage::Matching(matching_args) => { // Matching command
            println!("Matching only");
            println!("{:?}", matching_args);

            // Parse matching style

            crate::err_string("Matching not implemented yet".to_string())
        },
        args::RunStage::Full(full_args) => { // Full command
            println!("Full process");
            println!("{:?}", full_args);

            // Parse both styles

            crate::err_string("Full process not implemented yet".to_string())
        },
    }
}

/// [Stage 2.] TODO UNFINISHED FUNCTION
/// Run the process on the targets (layout, matching, or both).
/// Returns a `Result` with `()` or an `Err`.
pub fn run_process(targets: Targets) -> crate::Result<()> {

    // 2.1 Run the layout process
    if let Some(layout_style) = targets.layout_target {
        println!("#################");
        println!("Running layout...");
        println!("#################");
        println!("Layout style: {}", get_style_name(&layout_style));
        let layout_out = layout::do_layout(&layout_style, &targets.shared_args)?;
        layout::mesh_layout(&layout_out, &targets.shared_args)?;
    };

    // 2.2 Run MARIE between the two processes
    // TODO: Figure out MARIE interface

    // 2.3 Run the matching process
    #[allow(unused_variables)]
    if let Some(matching_network) = targets.matching_target {
        println!("###################");
        println!("Running matching...");
        println!("###################");
        let matching_out = matching::do_matching()?;
        matching::save_matching(&matching_out, &targets.shared_args)?;
    }

    Ok(())
}

/// Get the name of the layout style.
pub fn get_style_name(layout_style: &layout::LayoutStyle) -> String {
    layout::IsStyle::get_style_name(layout_style)
}

/// Top-level tests
#[cfg(test)]
mod tests {
    use super::*;
    use crate::layout::IsStyle;

    /// Test the `handle_cli_args` function for `layout` command.
    #[test]
    fn handle_layout_args() {
        let arguments = args::parse_cli_from(&["comrade", "layout", "-i", "input.stl", "-o", "output", "-l", "400", "iterative-circle", "-c", "8"]);
        let targets = handle_cli_args(arguments).unwrap();
        assert_eq!(targets.shared_args.input_path, "input.stl");
        assert_eq!(targets.shared_args.output_name, "output");
        assert_eq!(targets.shared_args.larmor_mhz, 400.0);
        assert_eq!(targets.layout_target.unwrap().get_style_name(), "Iterative Circle");
    }
}
