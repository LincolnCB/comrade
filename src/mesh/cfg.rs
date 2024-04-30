use crate::{
    args,
    io,
    mesh,
};
use crate::mesh::MeshMethodTrait;
use serde::{Serialize, Deserialize};

/// Mesh target struct. Includes the mesh method, method arguments, and general i/o arguments.
#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MeshTarget {
    /// Input path for the layout file (must be json).
    #[serde(default, alias = "input", alias = "in", alias = "i")]
    pub input_path: Option<String>,
    
    /// Output path for the mesh file (must match meshing method).
    #[serde(alias = "output", alias = "out", alias = "o")]
    pub output_path: String,
    
    /// Force save the mesh file, even if it's not the last stage targeted.
    #[serde(default, rename = "force_save")]
    pub save: bool,
    
    /// Meshing method.
    pub method: mesh::MethodEnum,
}
impl MeshTarget {
    /// Construct a mesh target from a config file.
    pub fn from_cfg_file(cfg_file: &str, is_first: bool, is_last: bool) -> args::ProcResult<Self> {
        let mut mesh_target: MeshTarget = io::load_deser_from(cfg_file)?;

        // Check the input path
        if is_first {
            if let Some(input_path) = mesh_target.input_path.as_ref() {
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

        let _ = crate::io::create(&format!("{}.{}", &mesh_target.output_path, mesh_target.method.get_output_extension()))?;

        mesh_target.save |= is_last;

        Ok(mesh_target)
    }
}
