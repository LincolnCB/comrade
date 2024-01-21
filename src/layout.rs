mod styles;
mod stl;
pub mod geo_3d;

use geo_3d::*;

use clap::Args;

// Re-export things from styles module
pub use styles::{
    LayoutStyleCliEnum,
    LayoutStyle,
    IsStyle,
};

/// A coil.
/// Contains a list of points.
#[derive(Debug)]
#[allow(dead_code)]
pub struct Coil {
    points: Vec<Point>,
    center: Point,
}

impl Coil {
    /// Create a new coil.
    pub fn new(points: Vec<Point>, center: Point) -> crate::Result<Self>{

        // Check if the coil is closed and ordered.
        let mut prev_point_id: usize = points.len() - 1;
        let mut prev_point;

        for (point_id, point) in points.iter().enumerate() {
            prev_point = &points[prev_point_id];
            if point.adj.len() != 2 {
                return crate::err_string("Coil point has wrong number of adjacent points".to_string());
            }
            if point.adj[0] != prev_point_id || prev_point.adj[1] != point_id  {
                return crate::err_string("Coil point has wrong adjacent points (out of order or unclosed)".to_string());
            }
            prev_point_id = point_id;
        }

        Ok(Coil{points, center})
    }
}

/// Layout struct.
/// This struct contains all the necessary results from the layout process.
/// Returned from the layout process, used as input to the matching process.
#[derive(Debug)]
pub struct Layout {
    pub coils: Vec<Coil>,
}

impl Layout {
    /// Create a new layout.
    pub fn new() -> Self{
        Layout{coils: Vec::new()}
    }
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
    // Load the STL file
    println!("Loading STL file...");
    let surface = stl::load_stl(&shared_args.input_path)?;

    // Run the layout style
    println!("Running layout style: {}...", layout_style.get_style_name());
    layout_style.do_layout(&surface)
}

/// Mesh the layout to output it to MARIE.
#[allow(unused_variables)]
pub fn mesh_layout(layout_out: &Layout, shared_args: &crate::args::SharedArgs) -> crate::Result<()>{
    println!("Dummy mesh_layout");
    crate::err_string("Meshing not implemented yet".to_string())
}
