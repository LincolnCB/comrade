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

use crate::{
    layout,
    args,
};

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

/// Layout methods enum.
/// To add a new method:
/// include it here,
/// add handling for its constructor in `LAYOUT_TARGET_CONSTRUCTION`,
/// and implement the `LayoutMethod` trait for it.
#[derive(Debug)]
#[enum_dispatch(LayoutMethod)]
pub enum LayoutChoice {
    /// Basic circular layout, based on Monika Sliwak's MATLAB prototype.
    SingleCircle(single_circle::Method),
    /// Manual circles layout, for specifying multiple circles by hand.
    ManualCircles(manual_circles::Method),
}

/// Layout construction array -- Written out in once place for easy modification.
/// To add a new method:
/// include it in the `LayoutChoice` enum,
/// add handling for its constructor here,
/// and implement the `LayoutMethod` trait for it.
const LAYOUT_TARGET_CONSTRUCTION: &[LayoutConstructor] = &[
    // EXAMPLE:
    // Single Circle layout constructor.
    LayoutConstructor{
        arg_name: "single_circle", 
        constructor: || {Ok(LayoutChoice::SingleCircle(single_circle::Method::new()?))},
    },
    // Manual Circles layout constructor.
    LayoutConstructor{
        arg_name: "manual_circles", 
        constructor: || {Ok(LayoutChoice::ManualCircles(manual_circles::Method::new()?))},
    },
];

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
    
    /// Parse the layout method argument file (allows different arguments for different methods).
    /// Takes a `&str` with the path to the argument file.
    fn parse_method_args(&mut self, arg_file: &str) -> args::ProcResult<()>;
    
    /// Run the layout process with the given arguments.
    /// Uses the `layout` module.
    /// Takes a loaded `Surface`.
    /// Returns a `ProcResult` with the `layout::Layout` or an `Err`.
    fn do_layout(&self, surface: &crate::layout::geo_3d::Surface) -> layout::ProcResult<layout::Layout>;
}

/// Layout constructor struct. Used to construct the layout methods from the arg_name string.
struct LayoutConstructor {
    /// Name of the layout method.
    arg_name: &'static str,
    /// Constructor function.
    constructor: fn() -> args::ProcResult<LayoutChoice>,
}

//
// ------------------------------------------------------------
// Functions and structs with no modification or reference needed
//      |
//      V
//

/// Layout target construction
impl LayoutChoice {
    /// Construct a layout method from a name (given in the config file).
    pub fn from_name(arg_name: &str) -> args::ProcResult<Self> {
        for constructor in LAYOUT_TARGET_CONSTRUCTION.iter() {
            if constructor.arg_name == arg_name {
                return (constructor.constructor)();
            }
        }

        // If the arg_name is not found, return an error with the available methods
        let mut error_str = format!("Layout method not found: {arg_name}\n");
        error_str.push_str("\n");
        error_str.push_str("Available methods:\n");
        for constructor in LAYOUT_TARGET_CONSTRUCTION.iter() {
            error_str.push_str(&format!("    {}\n", constructor.arg_name));
        }
        error_str.push_str("\n");
        error_str.push_str("New methods need to be added to src/layout/methods.rs");
        args::err_str(&error_str)
    }
}
