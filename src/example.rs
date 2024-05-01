use crate::{
    layout,
    mesh,
    sim,
    args,
    err_str,
    ComradeResult,
};
use strum::IntoEnumIterator;

/// Macro for constructing a full stage config from the method and stage.
macro_rules! construct_stage_cfg {
    (layout, $method:expr) => {
        layout::LayoutTarget{
            method: $method,
            input_path: "PATH/TO/INPUT/FILE".to_string(),
            output_path: Some("OPTIONAL/PATH/TO/OUTPUT/FILE".to_string()),
            save: false,
        }
    };
    (mesh, $method:expr) => {
        mesh::MeshTarget{
            method: $method,
            input_path: Some("OPTIONAL/PATH/TO/INPUT/FILE".to_string()),
            output_path: "PATH/TO/OUTPUT/FILE".to_string(),
            save: false,
        }
    };
    (sim, $method:expr) => {
        sim::SimTarget{
            method: $method,
            input_path: "PATH/TO/INPUT/FILE".to_string(),
            output_path: Some("OPTIONAL/PATH/TO/OUTPUT/FILE".to_string()),
            save: false,
        }
    };
    // (matching, $method:expr) => {
}

/// Macro for displaying the full config for a stage.
macro_rules! display_stage_cfg {
    ($stage:ident, $target_method_name:expr, $cfg_format:expr) => {
        let mut method_names = Vec::<String>::new();
        for method in $stage::MethodEnum::iter() {
            let method_name = serde_yaml::to_string(&method).unwrap().split_whitespace().collect::<Vec<_>>()[1].to_string();
            method_names.push(method_name);
        }
        let available_methods_str = format!("Available methods:\n{:#?}", method_names).replace(&['[', ']', ','][..], "");
        if $target_method_name.is_none() {
            println!("{}", available_methods_str);
            return Ok(());
        }
        match method_names.iter().enumerate().find(|&(_, name)| name == $target_method_name.as_ref().unwrap()) {
            Some(method_name) => {
                let method = $stage::MethodEnum::iter().nth(method_name.0).unwrap();
                let stage_cfg = construct_stage_cfg!($stage, method);
                match ($cfg_format) {
                    args::Format::Yaml => println!("{}", serde_yaml::to_string(&stage_cfg).unwrap()),
                    args::Format::Json => println!("{}", serde_json::to_string_pretty(&stage_cfg).unwrap()),
                    args::Format::Toml => println!("{}", toml::to_string_pretty(&stage_cfg).unwrap()),
                }
            },
            None => {
                return err_str(&format!("Method \"{}\" not found. {}", $target_method_name.unwrap(), available_methods_str));
            },
        };
    };
}

/// Display an example config file for a stage.
/// Returns a `ProcResult` with `()` or an `Err`.
pub fn display_config(example_args: args::ExampleArgs) -> ComradeResult<()> {
    match example_args.stage {
        args::RunStage::Layout => {
            display_stage_cfg!(layout, example_args.method, example_args.format);
        },
        args::RunStage::Mesh => {
            display_stage_cfg!(mesh, example_args.method, example_args.format);
        },
        args::RunStage::Sim => {
            display_stage_cfg!(sim, example_args.method, example_args.format);
        },
        args::RunStage::Match => {
            return err_str("Example config not yet implemented for this stage");
        },
    }
    Ok(())
}
