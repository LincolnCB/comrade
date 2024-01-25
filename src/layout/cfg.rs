use crate::args;
use crate::layout::LayoutChoice;
use serde::{Serialize, Deserialize};

/// Arguments for the layout process.
#[derive(Debug, Serialize, Deserialize)]
pub struct LayoutArgs {
    /// Layout method.
    #[serde(rename = "method")]
    pub method_name: String,

    /// Input path for the STL file.
    #[serde(alias = "input", alias = "in", alias = "i")]
    pub input_path: String,

    /// Output path for the layout file (must be json).
    #[serde(default, alias = "output", alias = "out", alias = "o")]
    pub output_path: Option<String>,

    /// Force save the layout file, even if it's not the last stage targeted.
    #[serde(default, rename = "force_save")]
    pub save: bool,
}

pub struct LayoutTarget {
    /// Layout method.
    pub layout_method: LayoutChoice,
    /// Layout arguments.
    pub layout_args: LayoutArgs,
}

impl LayoutTarget {
    /// Construct a layout target from a config file.
    #[allow(unused_variables)]
    pub fn from_cfg(cfg_file: &str, is_last: bool) -> args::ProcResult<Self> {
        let f = crate::io::open(cfg_file)?;
        let mut layout_args: LayoutArgs = serde_yaml::from_reader(f)?;
        
        let layout_method = LayoutChoice::from_name(&layout_args.method_name)?;

        if layout_args.save && layout_args.output_path.is_none() {
            args::err_str("Layout output path not specified, but force_save was set")?;
        }

        layout_args.save |= is_last;

        if layout_args.save && layout_args.output_path.is_none() {
            args::err_str("Layout output path not specified, but saving is required at the last stage")?;
        }

        if layout_args.save {
            let output_path = layout_args.output_path.as_ref().unwrap();
            if !output_path.ends_with(".json")
            {
                args::err_str("Layout output path must end with .json")?;
            }
            let _ = crate::io::create(output_path)?;
        }

        Ok(LayoutTarget{layout_method, layout_args})
    }
}

/// Private function to take hardcoded arg values and write the YAML file for it.
#[allow(dead_code)]
fn write_args_yaml(path: &str, layout_args: &LayoutArgs) -> args::ProcResult<()> {
    let f = crate::io::create(path)?;
    serde_yaml::to_writer(f, layout_args)?;
    Ok(())
}
