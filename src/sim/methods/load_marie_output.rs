use crate::{
    sim,
    args,
};

use sim::methods;

use serde::{Serialize, Deserialize};

/// Method struct for "simulation" by just loading previously calculated MARIE output.
/// This struct contains all the parameters needed to load a MARIE output file.
#[derive(Debug)]
pub struct Method {
    /// Arguments for the simulation method.
    method_args: MethodCfg,
}
impl Method {
    pub fn new() -> args::ProcResult<Self> {
        Ok(Method{method_args: MethodCfg::default()})
    }
}

/// Deserializer from yaml method cfg file
#[derive(Debug, Serialize, Deserialize)]
pub struct MethodCfg {
    /// Path to the MARIE output file.
    #[serde(alias = "mat_path")]
    marie_output_path: String,
}
impl Default for MethodCfg {
    fn default() -> Self {
        MethodCfg{
            marie_output_path: String::from(""),
        }
    }
}

impl methods::SimMethod for Method {
    /// Get the name of the simulation method.
    fn get_method_name(&self) -> String {
        String::from("Load MARIE output")
    }

    /// Parse the simulation method config file.
    fn parse_method_cfg(&mut self, method_cfg_file: &str) -> args::ProcResult<()> {
        let f = crate::io::open(method_cfg_file)?;
        self.method_args = serde_yaml::from_reader(f)?;
        Ok(())
    }

    /// Run the simulation process with the given arguments.
    fn do_simulation(&self) -> sim::ProcResult<sim::SimOutput> {
        sim::ProcResult::Ok(sim::SimOutput::new())
    }
}
