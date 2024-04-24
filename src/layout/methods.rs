/*!
 * This is the layout methods module.
 * Adding new methods should be done here.
 * 
 * New methods need:
 * - A struct implementing `LayoutMethod`
 * - An enum variant containing that struct in `LayoutChoice`
 * - A constructor arg_name and function in `LAYOUT_TARGET_CONSTRUCTION`
 * 
 */

use enum_dispatch::enum_dispatch;
use serde::{Serialize, Deserialize};

use crate::layout;

// Some helpful method examples
pub mod helper;

//
// ------------------------------------------------------------
// Code that requires modification to add a new layout method
//      |
//      V
//

// Source files for the layout methods
mod single_circle;
mod manual_circles;
mod iterative_circles;

/// Layout methods enum.
/// To add a new method:
/// include it here,
/// add handling for its constructor in `LAYOUT_TARGET_CONSTRUCTION`,
/// and implement the `LayoutMethod` trait for it.
#[derive(Serialize, Deserialize, Clone, Debug)]
#[enum_dispatch(LayoutMethod)]
#[serde(tag = "name", content = "args")]
pub enum LayoutChoice {
    /// Basic circular layout, based on Monika Sliwak's MATLAB prototype.
    #[serde(alias = "single_circle")]
    SingleCircle(single_circle::Method),
    /// Manual circles layout, for specifying multiple circles by hand.
    #[serde(alias = "manual_circles")]
    ManualCircles(manual_circles::Method),
    /// Iterative circles layout, for specifying multiple circles by hand and doing local optimization.
    #[serde(alias = "iterative_circles")]
    IterativeCircles(iterative_circles::Method),
}

//
// ------------------------------------------------------------
// Traits and structs that don't need modification,
// but are references for adding a new layout
//      |
//      V
//

/// Layout method trait.
/// This trait defines the functions that all layout methods must implement.
/// To add a new method:
/// include it in the `LayoutChoice` enum,
/// add handling for its constructor in `LAYOUT_TARGET_CONSTRUCTION`,
/// and implement this trait for it.
#[enum_dispatch] // This is a macro that allows the enum to be used in a trait object-like way
pub trait LayoutMethod {
    /// Get the arg_name of the layout method.
    fn get_method_name(&self) -> String;
    
    /// Run the layout process with the given arguments.
    /// Uses the `layout` module.
    /// Takes a loaded `Surface`.
    /// Returns a `ProcResult` with the `layout::Layout` or an `Err`.
    fn do_layout(&self, surface: &crate::geo_3d::Surface) -> layout::ProcResult<layout::Layout>;
}
