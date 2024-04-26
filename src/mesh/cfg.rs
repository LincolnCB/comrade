use crate::args;
use crate::mesh::{
    MeshChoice,
    MeshMethod,
};
use serde::{Serialize, Deserialize};

/// Arguments for the mesh process.
#[derive(Debug, Serialize, Deserialize)]
pub struct MeshArgs {
    /// Meshing method.
    pub method: MeshChoice,

    /// Input path for the layout file (must be json).
    #[serde(default, alias = "input", alias = "in", alias = "i")]
    pub input_path: Option<String>,

    /// Output path for the mesh file (must match meshing method).
    #[serde(alias = "output", alias = "out", alias = "o")]
    pub output_path: String,

    /// Force save the mesh file, even if it's not the last stage targeted.
    #[serde(default, rename = "force_save")]
    pub save: bool,
}

pub struct MeshTarget {
    /// Meshing method.
    pub mesh_method: MeshChoice,
    /// Meshing arguments.
    pub mesh_args: MeshArgs,
}

impl MeshTarget {
    /// Construct a mesh target from a config file.
    pub fn from_cfg_file(cfg_file: &str, is_first: bool, is_last: bool) -> args::ProcResult<Self> {
        let f = crate::io::open(cfg_file)?;
        let mut mesh_args: MeshArgs = serde_yaml::from_reader(f)?;
        
        // TODO: Refactor Target to clean this up, temporary
        let mesh_method = mesh_args.method.clone();

        // Check the input path
        if is_first {
            if let Some(input_path) = mesh_args.input_path.as_ref() {
                if !input_path.ends_with(".json")
                {
                    args::err_str("Mesh input path must end with .json")?;
                }
                let _ = crate::io::open(input_path)?;
            }
            else {
                args::err_str("Mesh input path not specified, but input path is required at the first stage")?;
            }
        }

        let _ = crate::io::create(&format!("{}.{}", &mesh_args.output_path, mesh_method.get_output_extension()))?;

        mesh_args.save |= is_last;

        Ok(MeshTarget{mesh_method, mesh_args})
    }
}

/// Private function to take hardcoded arg values and write the YAML file for it.
#[allow(dead_code)]
fn write_args_yaml(path: &str, mesh_args: &MeshArgs) -> args::ProcResult<()> {
    let f = crate::io::create(path)?;
    serde_yaml::to_writer(f, mesh_args)?;
    Ok(())
}
