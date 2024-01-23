/*!
 * This is the layout methods module.
 * Adding new methods should be done here.
 * 
 * New methods need:
 * - A struct implementing `LayoutMethod`
 * - An enum variant containing that struct in `LayoutChoice`
 * - A constructor name and function in `LAYOUT_TARGET_CONSTRUCTION`
 * 
 */

mod iterative_circle;

use enum_dispatch::enum_dispatch;

use crate::{
    layout,
    args,
};

/// Layout method trait.
/// This trait defines the functions that all layout methods must implement.
/// To add a new method:
/// implement this trait for it,
/// include it in the `LayoutChoice` enum,
/// and add handling for its constructor in `LAYOUT_TARGET_CONSTRUCTION`.
#[enum_dispatch] // enum dispatch allows us to use the enum as a kind of trait object
pub trait LayoutMethod {
    /// Get the name of the layout method.
    fn get_method_name(&self) -> String;

    /// Parse the layout method argument file (allows different arguments for different methods).
    /// Takes a `&str` with the path to the argument file.
    fn parse_method_args(&mut self, arg_file: &str) -> args::Result<()>;

    /// Run the layout process with the given arguments.
    /// Uses the `layout` module.
    /// Takes a loaded `Surface`.
    /// Returns a `Result` with the `layout::Layout` or an `Err`.
    fn do_layout(&self, surface: &crate::layout::geo_3d::Surface) -> layout::Result<layout::Layout>;
}

/// Layout methods enum.
/// To add a new method:
/// implement the `LayoutMethod` trait for it,
/// include it here,
/// and add handling for its constructor in `LAYOUT_TARGET_CONSTRUCTION`.
#[derive(Debug)]
#[enum_dispatch(LayoutMethod)]
pub enum LayoutChoice {
    /// Basic circular layout, based on Monika Sliwak's MATLAB prototype.
    IterativeCircle(iterative_circle::Method),
}

/// Layout construction array -- Laid out here for easy modification.
/// To add a new method:
/// implement the `LayoutMethod` trait for it,
const LAYOUT_TARGET_CONSTRUCTION: &[LayoutConstructor] = &[
    // Example layout constructor.
    LayoutConstructor{
        name: "iterative_circle", 
        constructor: || {Ok(LayoutChoice::IterativeCircle(iterative_circle::Method::new()?))},
    },
];

//
// ----------------------------------
// Private functions and structs, no modifications needed
//      |
//      V
//

/// Layout constructor struct. Used to construct the layout methods from the name string.
struct LayoutConstructor {
    name: &'static str,
    constructor: fn() -> args::Result<LayoutChoice>,
}

/// Layout target construction
impl LayoutChoice {
    /// Construct a layout method from a commandline name.
    pub fn from_name(name: &str) -> args::Result<Self> {
        for constructor in LAYOUT_TARGET_CONSTRUCTION.iter() {
            if constructor.name == name {
                return (constructor.constructor)();
            }
        }

        // If the name is not found, return an error with the available methods
        let mut error_str = format!("Layout method not found: {name}\n");
        error_str.push_str("Available methods: ");
        for constructor in LAYOUT_TARGET_CONSTRUCTION.iter() {
            error_str.push_str(constructor.name);
            error_str.push_str("\n");
        }
        args::err_str(&error_str)
    }
}
