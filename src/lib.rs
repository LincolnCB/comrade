pub mod layout;
pub mod matching;
pub mod args;

use args::{
    ComradeCli,
    RunStage,
};
use clap::Parser;

/// Targets struct.
/// This struct contains the layout and matching targets to do.
pub struct Targets{
    pub layout_style: Option<layout::LayoutStyle>,
    pub matching_style: Option<u32>, // TODO THIS IS A DUMMY
    pub shared_args: args::SharedArgs,
}

/// Parse the command line arguments for the comrade binary.
/// Uses the `clap` crate.
/// Returns a `Result` with a `comrade::RunArgs` struct or an `Err`.
pub fn parse_cli_args() -> Result<Targets, String>{
    let cli_args : ComradeCli = ComradeCli::parse();

    match cli_args.sub_command {
        RunStage::Layout(layout_cli) => {  
            println!("Layout only");
            println!("{:?}", layout_cli);

            let (layout_style, shared_args) = layout_cli.reconstruct()?;
            Ok(Targets{
                layout_style: Some(layout_style),
                matching_style: None,
                shared_args,
            })
            
        },
        RunStage::Matching(matching_args) => {
            println!("Matching only");
            println!("{:?}", matching_args);

            // Parse matching style

            Err("Matching not implemented yet".to_string())
        },
        RunStage::Full(full_args) => {
            println!("Full process");
            println!("{:?}", full_args);

            // Parse both styles
            Err("Full process not implemented yet".to_string())
        },
    }
}

/// Run the layout process with the given arguments.
/// Uses the `layout` module.
/// Takes parsed arguments (from `parse_layout_args` or future GUI).
/// Returns a `Result` with an `Option` containing the `layout::Layout` or an `Err`.
/// If the `Option` is `None`, the layout was saved to an output file (may have been saved AND returned)
pub fn do_layout() {
    println!();
    println!("Dummy layout");
    layout::load_stl();
}

/// Run the matching process with the given arguments.
/// Uses the `matching` module.
/// Takes parsed arguments (from `parse_matching_args`, mid-program variables, or future GUI).
/// Returns a `Result` with an `Option` containing the `matching::Matching` or an `Err`.
/// If the `Option` is `None`, the matching was saved to an output file (may have been saved AND returned)
pub fn do_matching() {
    println!();
    println!("Dummy matching");
    matching::load_coil_params();
}
