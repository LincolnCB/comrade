use crate::{
    args,
    io,
    sim
};
use crate::sim::SimMethodTrait;
use serde::{Serialize, Deserialize};

/// Simulation target struct. Includes the simulation method and method arguments, as well as general i/o arguments.
#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SimTarget {
    /// Input path for the simulation file.
    #[serde(default, alias = "input", alias = "in", alias = "i")]
    pub input_path: String,
    
    /// Output path for the simulation results (must be json).
    #[serde(alias = "output", alias = "out", alias = "o")]
    pub output_path: Option<String>,
    
    /// Force save the simulation results, even if it's not the last stage targeted.
    #[serde(default, rename = "force_save")]
    pub save: bool,

    /// Simulation method.
    pub method: sim::MethodEnum,
}
impl SimTarget {
    /// Construct a simulation target from a config file.
    pub fn from_cfg_file(cfg_file: &str, is_last: bool) -> args::ProcResult<Self> {
        let mut sim_target: SimTarget = io::load_deser_from(cfg_file)?;

        // Check that the input path is a supported filetype
        let mut supported = false;
        for filetype in sim_target.method.get_input_filetypes() {
            if sim_target.input_path.ends_with(filetype) {
                supported = true;
                break;
            }
        }
        if !supported {
            args::err_str("Input file type not supported by layout method")?;
        }

        // Check the output path
        if sim_target.save && sim_target.output_path.is_none() {
            args::err_str("Simulation output path not specified, but force_save was set")?;
        }

        sim_target.save |= is_last;

        if sim_target.save {
            if let Some(output_path) = sim_target.output_path.as_ref() {
                if !output_path.ends_with(".json")
                {
                    args::err_str("Simulation output path must end with .json")?;
                }
                let _ = crate::io::create(output_path)?;
            }
            else {
                args::err_str("Simulation output path not specified, but saving is required at the last stage")?;
            }
        }

        Ok(sim_target)
    }
}
