use clap::Args;

/// Load coil parameter file from the input path (.JSON file).
/// Uses the `serde_json`` crate.
/// Returns a `Result` with the `matching::coil` struct or an `Error`
pub fn load_coil_params() {
    println!("Dummy load_coil_params");
}

#[derive(Debug, Args)]
pub struct MatchingArgs {

    // TODO: Add options, add parsing to options
    #[arg(short, long)]
    /// Network type for the matching network.
    pub network_type: String,

    // TODO: Add parameters per network type to optimize, add a default value to each

}
