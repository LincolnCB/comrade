mod networks;

use clap::Args;

/// Arguments for the matching process.
/// Uses clap-derive.
#[derive(Debug, Args)]
pub struct MatchingArgs {

    // TODO: Add options, add parsing to options
    #[arg(short, long)]
    /// Network type for the matching network.
    pub network_type: String,

    // TODO: Add parameters per network type to optimize, add a default value to each
    
}

/// Matching struct.
/// This struct contains all the necessary results from the matching process.
/// Returned from the matching process.
#[derive(Debug)]
pub struct Matching {

}

/// Run the matching process.
/// Returns a `Result` with the `Matching` or an `Err`.
pub fn do_matching() -> crate::Result<Matching> {
    load_coil_params();
    println!("Dummy do_matching on network type");
    crate::err_string("Matching not implemented yet (matching.rs)".to_string())
}

/// Load coil parameter file from the input path (.JSON file).
/// Uses the `serde_json`` crate.
/// Returns a `Result` with the `matching::coil` struct or an `Err`
pub fn load_coil_params() {
    println!("Dummy load_coil_params");
}

/// Save the matching results to a file.
#[allow(unused_variables)]
pub fn save_matching(matching_out : &Matching, shared_args: &crate::args::SharedArgs) -> crate::Result<()>{
    println!("Dummy save_matching");
    crate::err_string("Meshing not implemented yet".to_string())
}
