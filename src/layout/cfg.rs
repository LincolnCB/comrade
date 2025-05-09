use crate::{
    args,
    io,
    layout,
};
use layout::LayoutMethodTrait;
use serde::{Serialize, Deserialize};

/// Layout target struct. Includes the layout method, method arguments, and general i/o arguments.
#[derive(Debug)]
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LayoutTarget {   
    /// Input path for the STL file.
    #[serde(alias = "input", alias = "in", alias = "i")]
    pub input_path: String,
    
    /// Output path for the layout file (must be json).
    #[serde(default, alias = "output", alias = "out", alias = "o")]
    pub output_path: Option<String>,

    /// Force save the layout file, even if it's not the last stage targeted.
    #[serde(default, rename = "force_save")]
    pub save: bool,

    /// Layout method.
    pub method: layout::MethodEnum,
}
impl LayoutTarget {
    /// Construct a layout target from a config file.
    pub fn from_cfg_file(cfg_file: &str, is_last: bool) -> args::ProcResult<Self> {
        let mut layout_target: LayoutTarget = io::load_deser_from(cfg_file)?;

        // Check that the input path is a supported filetype
        let mut supported = false;
        for filetype in layout_target.method.get_input_filetypes() {
            if layout_target.input_path.ends_with(filetype) {
                supported = true;
                break;
            }
        }
        if !supported {
            args::err_str("Input file type not supported by layout method")?;
        }

        // Check the output path
        if layout_target.save && layout_target.output_path.is_none() {
            args::err_str("Layout output path not specified, but force_save was set")?;
        }

        layout_target.save |= is_last;

        if layout_target.save {
            if let Some(output_path) = layout_target.output_path.as_ref() {
                if !output_path.ends_with(".json")
                {
                    args::err_str("Layout output path must end with .json")?;
                }
                let _ = crate::io::create(output_path)?;
            }
            else {
                args::err_str("Layout output path not specified, but saving is required at the last stage")?;
            }
        }

        Ok(layout_target)
    }
}
