pub mod geo_3d;
mod proc_errors;
mod cfg;
mod methods;
mod stl;

use geo_3d::*;

// Re-export errors
pub use proc_errors::{
    LayoutError,
    ProcResult,
    err_str,
};
// Re-export cfg handling
pub use cfg::{
    LayoutArgs,
    LayoutTarget,
};
// Re-export things from methods module
pub use methods::{
    LayoutChoice,
    LayoutMethod,
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
    pub fn new(points: Vec<Point>, center: Point) -> ProcResult<Self>{

        // Check if the coil is closed and ordered.
        let mut prev_point_id: usize = points.len() - 1;
        let mut prev_point;

        for (point_id, point) in points.iter().enumerate() {
            prev_point = &points[prev_point_id];
            if point.adj.len() != 2 {
                err_str("Coil point has wrong number of adjacent points")?;
            }
            if point.adj[0] != prev_point_id || prev_point.adj[1] != point_id  {
                err_str("Coil point has wrong adjacent points (out of order or unclosed)")?;
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

/// Run the layout process.
/// Returns a `ProcResult` with the `Layout` or an `Err`.
pub fn do_layout(layout_target: &LayoutTarget) -> ProcResult<Layout> {
    
    // Extract the information from the layout target
    let layout_method = &layout_target.layout_method;
    let layout_args = &layout_target.layout_args;

    println!("Layout method: {}", layout_method.get_method_name());

    // Load the STL file
    println!("Loading STL file...");
    let surface = stl::load_stl(&layout_args.input_path)?;

    // Run the layout method
    println!("Running layout method: {}...", layout_method.get_method_name());
    layout_method.do_layout(&surface)
}

pub fn save_layout(layout: &Layout, output_path: &str) -> ProcResult<()> {
    println!("Saving layout to {}...", output_path);
    // TODO: serde to YAML
    Ok(())
}
