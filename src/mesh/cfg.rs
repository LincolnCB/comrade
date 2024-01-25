use crate::args;
use serde::{Serialize, Deserialize};
use crate::layout::Layout;

/// Arguments for the mesh process.
#[derive(Debug, Serialize, Deserialize)]
pub struct MeshArgs {
    /// Input path for the layout file (must be json).
    #[serde(default, alias = "input", alias = "in", alias = "i")]
    pub input_path: Option<String>,

    /// Output path for the mesh file (must match meshing method).
    #[serde(default, alias = "output", alias = "out", alias = "o")]
    pub output_path: Option<String>,

    /// Force save the mesh file, even if it's not the last stage targeted.
    #[serde(default, rename = "force_save")]
    pub save: bool,
}
