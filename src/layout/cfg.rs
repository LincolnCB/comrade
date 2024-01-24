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

    /// Output path for the layout file (filetype will be attached).
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
        let f = std::fs::File::open(cfg_file)?;
        let mut layout_args: LayoutArgs = serde_yaml::from_reader(f)?;

        println!("{:?}", layout_args);
        
        // // TODO: Remove hardcoded shortcircuit
        // let mut layout_args = LayoutArgs{
        //     method_name: "iterative_circle".to_string(),
        //     input_path: "tests/data/tiny_cap_remesh.stl".to_string(),
        //     output_path: Some("tests/data/tiny_cap_remesh_layout".to_string()),
        //     save: false,
        // };
        // // End of hardcoded shortcircuit
        let layout_method = LayoutChoice::from_name(&layout_args.method_name)?;

        if layout_args.save && layout_args.output_path.is_none() {
            args::err_str("Layout output path not specified, but force_save was set")?;
        }

        layout_args.save |= is_last;

        if layout_args.save && layout_args.output_path.is_none() {
            args::err_str("Layout output path not specified, but saving is required at the last stage")?;
        }

        Ok(LayoutTarget{layout_method, layout_args})
    }
}

/// Private function to take hardcoded arg values and write the YAML file for it.
#[allow(dead_code)]
fn write_args_yaml(path: &str, layout_args: &LayoutArgs) -> args::ProcResult<()> {
    let f = std::fs::File::create(path)?;
    serde_yaml::to_writer(f, layout_args)?;
    Ok(())
}
