mod styles;
mod stl;

use clap::Args;

// Re-export things from styles module
pub use styles::{
    LayoutStyleCliEnum,
    LayoutStyle,
    IsStyle,
};

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

/// Run the layout process.
/// Returns a `Result` with the `Layout` or an `Err`.
pub fn do_layout(layout_style: &LayoutStyle, shared_args: &crate::args::SharedArgs) -> Result<Layout, String> {
    load_stl();
    layout_style.do_layout()
}

/// Load a STL file from the inut path.
/// Uses the `stl_io` crate.
/// Returns a `Result` with the `Mesh` or an `Err`
fn load_stl() {
    println!("Dummy load_stl");
}

/// Mesh the layout to output it to MARIE.
pub fn mesh_layout(layout_out: &Layout, shared_args: &crate::args::SharedArgs) -> Result<(), String>{
    println!("Dummy mesh_layout");
    Err("Meshing not implemented yet".to_string())
}
