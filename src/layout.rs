mod styles;

use clap::Args;

// Re-export the styles and CLI enum
pub use styles::{
    LayoutStyleCliEnum,
    LayoutStyle,
};

/// Load a STL file from the inut path.
/// Uses the `stl_io` crate.
/// Returns a `Result` with the `Mesh` or an `Err`
pub fn load_stl() {
    println!("Dummy load_stl");
}

/// Arguments for the layout process.
/// Uses clap-derive.
#[derive(Debug, Args)]
pub struct LayoutArgs {

    #[arg(short, long = "count")]
    /// Coil count for the layout.
    pub coil_count: u64,

    #[arg(short, long = "mesh")]
    /// Output a mesh file with the same output name.
    pub mesh: bool,

    // TODO: Inductive decoupling, default 11dB
}

/// Layout struct.
/// This struct contains all the necessary results from the layout process.
/// Returned from the layout process, used as input to the matching process.
#[derive(Debug)]
pub struct Layout {

}
