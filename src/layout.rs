mod styles;

use clap::Args;

// Re-export the styles CLI enum
pub use styles::LayoutStyleCliEnum;

/// Load a STL file from the inut path.
/// Uses the `stl_io` crate.
/// Returns a `Result` with the `Mesh` or an `Error`
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
    
    // #[clap(arg_enum)]
    // /// Layout style to use.
    // pub style: styles::LayoutStyleCliEnum,

    // TODO: Inductive decoupling, default 11dB

}

/// Layout struct.
/// This struct contains all the necessary results from the layout process.
/// Returned from the layout process, used as input to the matching process.
pub struct Layout {

}
