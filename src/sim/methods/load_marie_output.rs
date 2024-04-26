use crate::sim;

use sim::methods;

use serde::{Serialize, Deserialize};

/// Method struct for "simulation" by just loading previously calculated MARIE output.
/// This struct contains all the parameters needed to load a MARIE output file.
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct Method {
    // No fields yet
}
impl Default for Method {
    fn default() -> Self {
        Method{
            // No fields yet
        }
    }
}

impl methods::SimMethodTrait for Method {
    /// Get the name of the simulation method.
    fn get_method_name(&self) -> &'static str {
        "Load MARIE MAT Output"
    }

    /// Get a vector of viable input filetypes for the simulation method.
    fn get_input_filetypes(&self) -> Vec<&'static str> {
        vec!["mat"]
    }

    /// Run the simulation process with the given arguments.
    fn do_simulation(&self) -> sim::ProcResult<sim::SimOutput> {

        // TODO: do more of this
        // // Load the MARIE output file
        // let f = crate::io::open(&self.method_args.marie_output_path)?;

        Ok(sim::SimOutput::new())
    }
}
