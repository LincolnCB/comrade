use crate::args;
use crate::sim::{
    SimChoice,
    SimMethod,
};
use serde::{Serialize, Deserialize};

/// Arguments for the simulation process.
#[derive(Debug, Serialize, Deserialize)]
pub struct SimArgs {
    /// Simulation method.
    #[serde(rename = "method")]
    pub method_name: String,

    /// Simulation method method_cfg.
    pub method_cfg: String,
}

/// Simulation target struct.
/// Contains the simulation method and arguments.
pub struct SimTarget {
    /// Simulation method.
    pub sim_method: SimChoice,
    /// Simulation arguments.
    pub sim_args: SimArgs,
}
impl SimTarget {
    /// Construct a simulation target from a config file.
    pub fn from_argfile(cfg_file: &str) -> args::ProcResult<Self> {
        let f = crate::io::open(cfg_file)?;
        let sim_args: SimArgs = serde_yaml::from_reader(f)?;
        
        let mut sim_method = SimChoice::from_name(&sim_args.method_name)?;

        // Parse the method_cfg file
        sim_method.parse_method_cfg(&sim_args.method_cfg)?;

        Ok(SimTarget{
            sim_method,
            sim_args,
        })
    }
}

/// Private function to take hardcoded arg values and write the YAML file for it.
#[allow(dead_code)]
fn write_args_yaml(path: &str, sim_args: &SimArgs) -> args::ProcResult<()> {
    let f = crate::io::create(path)?;
    serde_yaml::to_writer(f, sim_args)?;
    Ok(())
}


