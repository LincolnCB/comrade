mod proc_errors;
mod cfg;
mod methods;

use serde::{Serialize, Deserialize};

pub use proc_errors::{
    SimError,
    ProcResult,
    err_str,
};
// Re-export cfg handling
pub use cfg::SimTarget;
// Re-export simulation methods
pub use methods::{
    MethodEnum,
    SimMethodTrait,
};

/// Simulation output struct.
/// This struct contains all the necessary results from the simulation process.
/// Returned from the simulation process, used as input to the matching process.
#[derive(Debug)]
#[derive(Serialize, Deserialize)]
pub struct SimOutput {
    pub coil_values: Vec<CoilRLC>,
}
impl SimOutput {
    /// Create a new simulation.
    pub fn new() -> Self{
        SimOutput{coil_values: Vec::new()}
    }
}

/// Coil RLC values struct.
/// This struct contains the resistance, inductance, and capacitance values for a coil.
#[derive(Debug)]
#[derive(Serialize, Deserialize)]
pub struct CoilRLC {
    pub resistance: f64,
    pub inductance: f64,
    pub capacitance: f64,
}
impl CoilRLC {
    pub fn rlc(&self) -> (f64, f64, f64) {
        (self.resistance, self.inductance, self.capacitance)
    }
}

pub fn do_simulation(sim_target: &SimTarget) -> ProcResult<SimOutput> {

    // Extract the simulation method and arguments from target
    let sim_method = &sim_target.method;

    println!("Simulating...");

    // Run the simulation method
    println!("Running simulation method: {}", sim_method.get_method_display_name());
    sim_method.do_simulation()
}

