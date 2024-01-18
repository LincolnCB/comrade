mod styles;
mod stl;

use clap::Args;

// Re-export things from styles module
pub use styles::{
    LayoutStyleCliEnum,
    LayoutStyle,
    IsStyle,
};

/// The substrate surface.
/// Contains a list of points.
#[derive(Debug)]
pub struct Surface {
    pub points: Vec<Point>,
}

/// A point in 3D space.
/// Contains the coordinates and a list of adjacent points.
/// The adjacent points are stored as indices in the `Surface` struct.
/// Adjacent points are found from the triangles in the STL file.
#[derive(Debug)]
pub struct Point {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub adj: Vec<usize>,
    // TODO: Maybe include averaged normal vector? Maybe one point per triangle?
}


/// Layout struct.
/// This struct contains all the necessary results from the layout process.
/// Returned from the layout process, used as input to the matching process.
#[derive(Debug)]
pub struct Layout {

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

/// Run the layout process.
/// Returns a `Result` with the `Layout` or an `Err`.
#[allow(unused_variables)]
pub fn do_layout(layout_style: &LayoutStyle, shared_args: &crate::args::SharedArgs) -> crate::Result<Layout> {
    let surface = stl::load_stl(&shared_args.input_path)?;
    layout_style.do_layout()
}

/// Mesh the layout to output it to MARIE.
#[allow(unused_variables)]
pub fn mesh_layout(layout_out: &Layout, shared_args: &crate::args::SharedArgs) -> crate::Result<()>{
    println!("Dummy mesh_layout");
    crate::err_string("Meshing not implemented yet".to_string())
}
