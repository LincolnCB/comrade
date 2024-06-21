/*!
 * This is the layout methods module.
 * Adding new methods should be done here.
 * 
 * New methods need:
 * - A struct implementing `LayoutMethodTrait`
 * - An enum variant containing that struct in `MethodEnum`
 * 
 */

use enum_dispatch::enum_dispatch;
use serde::{Serialize, Deserialize};
use strum::EnumIter;

use crate::layout;

// Some helpful method examples
pub mod helper;

//
// ------------------------------------------------------------
// Code that requires modification to add a new layout method
//      |
//      V
//

// Add the source module for the layout methods here
mod gradient_circles;
mod alternating_circles;
mod k_means_isometric;

/// Layout methods enum.
/// To add a new method:
/// include it here
/// and make sure the source implements the `LayoutMethodTrait` trait.
#[derive(Debug)]
#[derive(Serialize, Deserialize)]
#[derive(EnumIter)]
#[enum_dispatch(LayoutMethodTrait)]
#[serde(tag = "name", content = "args")]
pub enum MethodEnum {

    /// Gradient-based circles layout, using gradient descent to optimize circle positions.
    #[serde(rename = "gradient_circles")]
    GradientCircles(gradient_circles::Method),

    /// Alternating symmetric circles layout, alternating between radius and position optimization with symmetry.
    #[serde(rename = "alternating_circles")]
    AlternatingCircles(alternating_circles::Method),

    /// K-means isometric layout, for clustering points and creating circles from the clusters.
    #[serde(rename = "k_means_isometric")]
    KMeansIsometric(k_means_isometric::Method),
}

//
// ------------------------------------------------------------
// The trait doesn't need modification,
// but needs to be implemented in each method module
//      |
//      V
//

/// Layout method trait.
/// This trait defines the functions that all layout methods must implement.
/// To add a new method:
/// include it in the `MethodEnum` enum
/// and make sure it implements this trait
#[enum_dispatch] // This is a macro that allows the enum to be used in a trait-object-like way
pub trait LayoutMethodTrait {

    /// Get the name of the layout method.
    fn get_method_display_name(&self) -> &'static str;

    /// Get a vector of viable input filetypes for the layout method.
    /// Defaults to STL.
    fn get_input_filetypes(&self) -> Vec<&'static str> {
        vec!["stl"]
    }

    /// Load the layout input file. 
    /// Default implementation is for STL files.
    fn load_surface(&self, input_path: &str) -> layout::ProcResult<crate::geo_3d::Surface> {
        println!("Loading STL file: {}", input_path);
        Ok(crate::io::stl::load_stl(input_path)?)
    }
    
    /// Run the layout process with the given arguments.
    /// Uses the `layout` module.
    /// Takes a loaded `Surface`.
    /// Returns a `ProcResult` with the `layout::Layout` or an `Err`.
    fn do_layout(&self, surface: &crate::geo_3d::Surface) -> layout::ProcResult<layout::Layout>;
}
