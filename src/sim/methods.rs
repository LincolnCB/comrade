/*!
 * This is the simulation methods module.
 * Adding new methods should be done here.
 * 
 * New methods need:
 * - A struct implementing `SimMethodTrait`
 * - An enum variant containing that struct in `MethodEnum`
 * 
 */

use enum_dispatch::enum_dispatch;
use serde::{Serialize, Deserialize};
use strum::EnumIter;

use crate::sim;

//
// ------------------------------------------------------------
// Code that requires modification to add a new simulation method
//      |
//      V
//

// Add the source module for the layout methods here
mod load_marie_output;

/// Simulation methods enum.
/// To add a new method:
/// include it here
/// and make sure the source implements the `SimMethodTrait` trait.
#[derive(Debug, Clone)]
#[derive(Serialize, Deserialize)]
#[derive(EnumIter)]
#[enum_dispatch(SimMethodTrait)]
#[serde(tag = "name", content = "args")]
pub enum MethodEnum {

    /// Direct loading of MARIE simulation output, where the simulation was already done.
    #[serde(rename = "load_marie_output")]
    LoadMarieOutput(load_marie_output::Method),
}

//
// ------------------------------------------------------------
// The trait doesn't need modification,
// but needs to be implemented in each method module
//      |
//      V
//

/// Sim method trait.
/// This trait defines the functions that all simulation methods must implement.
/// To add a new method:
/// include it in the `MethodEnum` enum
/// and make sure it implements this trait.
#[enum_dispatch] // This is a macro that allows the enum to be used in a trait object-like way
pub trait SimMethodTrait {
    
    /// Get the arg_name of the simulation method.
    fn get_method_display_name(&self) -> &'static str;
    
    /// Get a vector of viable input filetypes for the simulation method.
    fn get_input_filetypes(&self) -> Vec<&'static str>;
    
    /// Run the simulation process with the given arguments.
    /// Uses the `sim` module.
    /// Returns a `ProcResult` with the `sim::SimOutput` or an `Err`.
    fn do_simulation(&self) -> sim::ProcResult<sim::SimOutput>;
}

