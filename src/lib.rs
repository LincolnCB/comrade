pub mod layout;
pub mod matching;
pub mod args;

use args::{
    ComradeCli,
    RunStage,
};
use clap::Parser;

/// Parse the command line arguments for the comrade binary.
/// Uses the `clap` crate.
/// Returns a `Result` with a `comrade::RunArgs` struct or an `Error`.
pub fn parse_cli_args() {
    let cli_args : ComradeCli = ComradeCli::parse();

    match cli_args.sub_command {
        RunStage::Layout(layout_args) => {  
            println!("Layout only");
            println!("{:?}", layout_args);
        },
        RunStage::Matching(matching_args) => {
            println!("Matching only");
            println!("{:?}", matching_args);
        },
        RunStage::Full(full_args) => {
            println!("Full process");
            println!("{:?}", full_args);
        },
    }
}

/// Run the layout process with the given arguments.
/// Uses the `layout` module.
/// Takes parsed arguments (from `parse_layout_args` or future GUI).
/// Returns a `Result` with an `Option` containing the `layout::Layout` or an `Error`.
/// If the `Option` is `None`, the layout was saved to an output file (may have been saved AND returned)
pub fn do_layout() {
    println!();
    println!("Dummy layout");
    layout::load_stl();
}

/// Run the matching process with the given arguments.
/// Uses the `matching` module.
/// Takes parsed arguments (from `parse_matching_args`, mid-program variables, or future GUI).
/// Returns a `Result` with an `Option` containing the `matching::Matching` or an `Error`.
/// If the `Option` is `None`, the matching was saved to an output file (may have been saved AND returned)
pub fn do_matching() {
    println!();
    println!("Dummy matching");
    matching::load_coil_params();
}
