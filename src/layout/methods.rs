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

mod iterative_circle;

use enum_dispatch::enum_dispatch;

use crate::{
    layout,
    args,
};

//
// ------------------------------------------------------------
// Code that requires modification to add a new layout method
//      |
//      V
//

/// Layout methods enum.
/// To add a new method:
/// include it here,
/// add handling for its constructor in `LAYOUT_TARGET_CONSTRUCTION`,
/// and implement the `LayoutMethod` trait for it.
#[derive(Debug)]
#[enum_dispatch(LayoutMethod)]
pub enum LayoutChoice {
    /// Basic circular layout, based on Monika Sliwak's MATLAB prototype.
    IterativeCircle(iterative_circle::Method),
}

/// Layout construction array -- Written out in once place for easy modification.
/// To add a new method:
/// include it in the `LayoutChoice` enum,
/// add handling for its constructor here,
/// and implement the `LayoutMethod` trait for it.
const LAYOUT_TARGET_CONSTRUCTION: &[LayoutConstructor] = &[
    // Example layout constructor.
    LayoutConstructor{
        arg_name: "iterative_circle", 
        constructor: || {Ok(LayoutChoice::IterativeCircle(iterative_circle::Method::new()?))},
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
#[enum_dispatch] // enum dispatch allows us to use the enum as a kind of trait object
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
    arg_name: &'static str,
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
    /// Construct a layout method from a commandline arg_name.
    pub fn from_name(arg_name: &str) -> args::ProcResult<Self> {
        for constructor in LAYOUT_TARGET_CONSTRUCTION.iter() {
            if constructor.arg_name == arg_name {
                return (constructor.constructor)();
            }
        }

        // If the arg_name is not found, return an error with the available methods
        let mut error_str = format!("Layout method not found: {arg_name}\n");
        error_str.push_str("Available methods: ");
        for constructor in LAYOUT_TARGET_CONSTRUCTION.iter() {
            error_str.push_str(constructor.arg_name);
            error_str.push_str("\n");
        }
        args::err_str(&error_str)
    }
}
