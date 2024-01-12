pub mod layout;
pub mod matching;

/// Struct to hold the parsed command line arguments.
/// Stores a `bool` for whether to run each of the layout process and matching process.
/// For each, stores `Option` containing the respective argument struct (`layout::LayoutArgs` or `matching::MatchingArgs`).
/// Used by `parse_cli_args` to return the parsed arguments.
/// Used by `main` to run `do_layout` and/or `do_matching`
pub struct RunArgs {
    pub run_layout: bool,
    pub layout_args: Option<layout::LayoutArgs>,
    pub run_matching: bool,
    pub matching_args: Option<matching::MatchingArgs>,
}

/// Parse the command line arguments for the comrade binary.
/// Uses the `clap` crate.
/// Returns a `Result` with a `comrade::RunArgs` struct or an `Error`.
pub fn parse_cli_args() {
    println!("");
    println!("Dummy parse_cli_args");
}

/// Run the layout process with the given arguments.
/// Uses the `layout` module.
/// Takes parsed arguments (from `parse_layout_args` or future GUI).
/// Returns a `Result` with an `Option` containing the `layout::Layout` or an `Error`.
/// If the `Option` is `None`, the layout was saved to an output file (may have been saved AND returned)
pub fn do_layout() {
    println!("");
    println!("Dummy layout");
    layout::load_stl();
}

/// Run the matching process with the given arguments.
/// Uses the `matching` module.
/// Takes parsed arguments (from `parse_matching_args`, mid-program variables, or future GUI).
/// Returns a `Result` with an `Option` containing the `matching::Matching` or an `Error`.
/// If the `Option` is `None`, the matching was saved to an output file (may have been saved AND returned)
pub fn do_matching() {
    println!("");
    println!("Dummy matching");
    matching::load_coil_params();
}
