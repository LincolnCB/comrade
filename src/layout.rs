mod methods;
mod stl;
pub mod geo_3d;

use crate::{
    args,
};
use geo_3d::*;

// Re-export things from methods module
pub use methods::{
    LayoutChoice,
    LayoutMethod,
};

/// Layout process error type.
#[derive(Debug)]
pub enum LayoutError {
    /// IO error.
    IoError(std::io::Error),
    /// StringOnly error.
    StringOnly(String),
}
impl std::fmt::Display for LayoutError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LayoutError::IoError(error) => write!(f, "IO Error:\n{}", error),
            LayoutError::StringOnly(error) => write!(f, "{}", error),
        }
    }
}
impl From<std::io::Error> for LayoutError {
    fn from(error: std::io::Error) -> Self {
        LayoutError::IoError(error)
    }
}
impl From<String> for LayoutError {
    fn from(error: String) -> Self {
        LayoutError::StringOnly(error)
    }
}

/// Result type for the `layout` crate.
pub type Result<T> = std::result::Result<T, LayoutError>;

/// Create a `LayoutError::StringOnly` from a string.
pub fn err_str<T>(error_str: &str) -> Result<T> {
    Err(LayoutError::StringOnly(error_str.to_string()))
}


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
    pub fn new(points: Vec<Point>, center: Point) -> Result<Self>{

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
    pub fn from_cfg(layout_cfg_file: &str) -> args::Result<Self> {
        // TODO: Remove hardcoded shortcircuit
        let layout_method = LayoutChoice::from_name("iterative_circle")?;
        let layout_args = LayoutArgs{input_path: "tests/data/tiny_cap_remesh.stl".to_string()};

        Ok(LayoutTarget{layout_method, layout_args})
    }
}

/// Run the layout process.
/// Returns a `Result` with the `Layout` or an `Err`.
pub fn do_layout(layout_target: &LayoutTarget) -> Result<Layout> {
    
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
