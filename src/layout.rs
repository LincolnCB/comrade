pub mod geo_3d;
mod proc_errors;
mod cfg;
mod methods;
mod stl;

use serde::{Serialize, Deserialize};

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
// Re-export layout methods
pub use methods::{
    LayoutChoice,
    LayoutMethod,
};

/// Layout struct.
/// This struct contains all the necessary results from the layout process.
/// Returned from the layout process, used as input to the matching process.
#[derive(Debug, Serialize, Deserialize)]
pub struct Layout {
    pub coils: Vec<Coil>,
}
impl Layout {
    /// Create a new layout.
    pub fn new() -> Self{
        Layout{coils: Vec::new()}
    }
}

/// A coil.
/// Contains a list of points.
#[derive(Debug, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct Coil {
    pub center: Point,
    pub normal: GeoVector,
    pub points: Vec<CoilPoint>,
}
impl Coil {
    /// Create a new coil.
    /// Points must be in order -- the coil will be closed automatically.
    pub fn new(
        center: Point,
        normal: GeoVector,
        points: Vec<Point>,
    ) -> ProcResult<Self>{

        // Check that there are at least 3 points
        if points.len() < 3 {
            err_str("Coil must have at least 3 points!")?;
        }

        // Connect the points
        let mut coil_points = Vec::<CoilPoint>::new();

        for (point_id, point) in points.iter().enumerate() {
            let next_id = (point_id + 1) % points.len();
            let prev_id = (point_id + points.len() - 1) % points.len();

            coil_points.push(CoilPoint{
                point: point.clone(),
                id: point_id,
                next_id,
                prev_id,
            });
        }

        Ok(Coil{center, normal, points: coil_points})
    }
}

/// A point on a coil (includes adjacency and surface vectors).
#[derive(Debug, Serialize, Deserialize)]
pub struct CoilPoint {
    pub point: Point,
    pub id: usize,
    pub next_id: usize,
    pub prev_id: usize,
}

/// Run the layout process.
/// Returns a `ProcResult` with the `Layout` or an `Err`.
pub fn do_layout(layout_target: &LayoutTarget) -> ProcResult<Layout> {
    
    // Extract the layout method and arguments from target
    let layout_method = &layout_target.layout_method;
    let layout_args = &layout_target.layout_args;

    // TODO: Handle different types of mesh files here.
    // Make sure to put all the optional filetype names in the cfg module.
    
    // Load the STL file
    println!("Loading STL file...");
    let surface = stl::load_stl(&layout_args.input_path)?;

    // Run the layout method
    println!("Running layout method: {}...", layout_method.get_method_name());
    layout_method.do_layout(&surface)
}

pub fn save_layout(layout: &Layout, output_path: &str) -> ProcResult<()> {
    println!("Saving layout to {}...", output_path);
    let f = crate::io::create(output_path)?;
    serde_json::to_writer_pretty(f, layout)?;
    Ok(())
}

pub fn load_layout(input_path: &str) -> ProcResult<Layout> {
    println!("Loading layout from {}...", input_path);
    let f = crate::io::open(input_path)?;
    let layout: Layout = serde_json::from_reader(f)?;
    Ok(layout)
}
