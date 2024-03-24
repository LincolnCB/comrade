/*!
 * This is the simulation methods module.
 * Adding new methods should be done here.
 * 
 * New methods need:
 * - A struct implementing `SimMethod`
 * - An enum variant containing that struct in `SimChoice`
 * - A constructor arg_name and function in `SIM_TARGET_CONSTRUCTION`
 * 
 */

use enum_dispatch::enum_dispatch;

use crate::{
    sim,
    args,
};

//
// ------------------------------------------------------------
// Code that requires modification to add a new simulation method
//      |
//      V
//

// Source files for the simulation methods
mod load_marie_output;

/// Simulation methods enum.
/// To add a new method:
/// include it here,
/// add handling for its constructor in `SIM_TARGET_CONSTRUCTION`,
/// and implement the `SimMethod` trait for it.
#[derive(Debug)]
#[enum_dispatch(SimMethod)]
pub enum SimChoice {
    /// Direct loading of MARIE simulation output, where the simulation was already done.
    LoadMarieOutput(load_marie_output::Method),
}

/// Simulation construction array -- Written out in once place for easy modification.
/// To add a new method:
/// include it in the `SimChoice` enum,
/// add handling for its constructor here,
/// and implement the `SimMethod` trait for it.
const SIM_TARGET_CONSTRUCTION: &[SimConstructor] = &[
    // EXAMPLE:
    // Direct MARIE output loading constructor.
    SimConstructor{
        arg_name: "load_marie_output", 
        constructor: || {Ok(SimChoice::LoadMarieOutput(load_marie_output::Method::new()?))},
    },
];

//
// ------------------------------------------------------------
// Traits and structs that don't need modification,
// but are references for adding a new simulation
//      |
//      V
//

/// Sim method trait.
/// This trait defines the functions that all simulation methods must implement.
/// To add a new method:
/// include it in the `SimChoice` enum,
/// add handling for its constructor in `SIM_TARGET_CONSTRUCTION`,
/// and implement this trait for it.
#[enum_dispatch] // This is a macro that allows the enum to be used in a trait object-like way
pub trait SimMethod {
    /// Get the arg_name of the simulation method.
    fn get_method_name(&self) -> String;
    
    /// Parse the simulation method config file (allows different arguments for different methods).
    /// Takes a `&str` with the path to the argument file.
    fn parse_method_cfg(&mut self, method_cfg_file: &str) -> args::ProcResult<()>;
    
    /// Run the simulation process with the given arguments.
    /// Uses the `sim` module.
    /// Returns a `ProcResult` with the `sim::SimOutput` or an `Err`.
    fn do_simulation(&self) -> sim::ProcResult<sim::SimOutput>;
}

/// Sim constructor struct. Used to construct the simulation methods from the arg_name string.
struct SimConstructor {
    /// Name of the simulation method.
    arg_name: &'static str,
    /// Constructor function.
    constructor: fn() -> args::ProcResult<SimChoice>,
}

//
// ------------------------------------------------------------
// Functions and structs with no modification or reference needed
//      |
//      V
//

/// Sim target construction
impl SimChoice {
    /// Construct a simulation method from a name (given in the config file).
    pub fn from_name(arg_name: &str) -> args::ProcResult<Self> {
        for constructor in SIM_TARGET_CONSTRUCTION.iter() {
            if constructor.arg_name == arg_name {
                return (constructor.constructor)();
            }
        }

        // If the arg_name is not found, return an error with the available methods
        let mut error_str = format!("Simulation method not found: {arg_name}\n");
        error_str.push_str("\n");
        error_str.push_str("Available methods:\n");
        for constructor in SIM_TARGET_CONSTRUCTION.iter() {
            error_str.push_str(&format!("    {}\n", constructor.arg_name));
        }
        error_str.push_str("\n");
        error_str.push_str("New methods need to be added to src/sim/methods.rs");
        args::err_str(&error_str)
    }
}
