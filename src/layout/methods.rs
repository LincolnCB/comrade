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
mod single_circle;
mod manual_circles;
mod manual_symmetric;
mod iterative_circles;

/// Layout methods enum.
/// To add a new method:
/// include it here
/// and make sure the source implements the `LayoutMethodTrait` trait.
#[derive(Serialize, Deserialize, Clone, Debug)]
#[derive(EnumIter)]
#[enum_dispatch(LayoutMethodTrait)]
#[serde(tag = "name", content = "args")]
pub enum MethodEnum {

    /// Basic circular layout, based on Monika Sliwak's MATLAB prototype.
    #[serde(rename = "single_circle")]
    SingleCircle(single_circle::Method),

    /// Manual circles layout, for specifying multiple circles by hand.
    #[serde(rename = "manual_circles")]
    ManualCircles(manual_circles::Method),

    /// Manual circles layout with symmetry plane. 
    #[serde(rename = "manual_symmetric")]
    ManualSymmetric(manual_symmetric::Method),

    /// Iterative circles layout, for specifying multiple circles by hand and doing local optimization.
    #[serde(rename = "iterative_circles")]
    IterativeCircles(iterative_circles::Method),
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
