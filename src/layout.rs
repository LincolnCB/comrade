pub mod geo_3d;
mod proc_errors;
mod methods;
mod stl;

use crate::{
    args,
};
use geo_3d::*;

// Re-export things from methods module
pub use methods::{
    LayoutChoice,
    LayoutMethod,
};
pub use proc_errors::{
    LayoutError,
    ProcResult,
    err_str,
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

/// Arguments for the layout process.
pub struct LayoutArgs {
    /// Input path for the STL file.
    pub input_path: String,
}

pub struct LayoutTarget {
    /// Layout method.
    pub layout_method: LayoutChoice,
    /// Layout arguments.
    pub layout_args: LayoutArgs,
}

impl LayoutTarget {
    /// Construct a layout target from a config file.
    #[allow(unused_variables)]
    pub fn from_cfg(layout_cfg_file: &str) -> args::ProcResult<Self> {
        // TODO: Remove hardcoded shortcircuit
        let layout_method = LayoutChoice::from_name("iterative_circle")?;
        let layout_args = LayoutArgs{input_path: "tests/data/tiny_cap_remesh.stl".to_string()};

        Ok(LayoutTarget{layout_method, layout_args})
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
