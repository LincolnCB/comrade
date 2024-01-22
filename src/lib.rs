pub mod layout;
pub mod matching;
pub mod args;

use std::io;
use clap;
use strum::IntoEnumIterator;

/// Targets struct.
/// This struct contains the layout and matching targets to do.
pub struct Targets{
    pub layout_target: Option<layout::LayoutTarget>,
    pub mesh_target: Option<()>, // TODO THIS IS A DUMMY
    pub sim_target: Option<()>, // TODO THIS IS A DUMMY
    pub matching_target: Option<()>, // TODO THIS IS A DUMMY
    pub shared_args: args::SharedArgs,
}

// TODO: Refactor error types
/// Result type for the `comrade` crate.
type Result<T> = std::result::Result<T, ComradeError>;

/// Error-type enum for the `comrade` crate.
/// Can handle errors from the `clap` crate and the `stl_io` crate.
/// Will handle other errors in the future.
#[derive(Debug)]
pub enum ComradeError {
    // TODO: Refactor error types
    IOError(io::Error),
    ClapError(clap::Error),
    Default(String),
}
impl From<io::Error> for ComradeError {
    fn from(error: io::Error) -> Self {
        ComradeError::IOError(error)
    }
}
impl From<clap::Error> for ComradeError {
    fn from(error: clap::Error) -> Self {
        ComradeError::ClapError(error)
    }
}
impl From<String> for ComradeError {
    fn from(error: String) -> Self {
        ComradeError::Default(error)
    }
}

/// Create a `Result` with an `Err` from a string.
/// Shorthand to avoid writing `Err(crate::ComradeError::Default(error_str))`.
pub fn err_string<T>(error_str: String) -> crate::Result<T> {
    Err(ComradeError::Default(error_str))
}
    

/// [Stage 1.]
/// Parse the command line arguments for the comrade binary.
/// Uses the `clap` crate.
/// Expects to see a start stage and an optional end stage that must come after the start
/// For each stage to be run between them, checks for a required corresponding config file.
/// Returns a `Result` with the `Targets` or an `Err`.
pub fn handle_cli_args(cli_args : args::ComradeCli) -> crate::Result<Targets>{
    let end_stage = if let Some(end_stage) = cli_args.end_stage {
        end_stage
    } else {
        cli_args.start_stage.clone()
    };

    if cli_args.start_stage.stage_num() > end_stage.stage_num() {
        return crate::err_string(format!("Start stage {} is after end stage {}", cli_args.start_stage, end_stage));
    }
    if cli_args.start_stage.stage_num() == end_stage.stage_num() {
        println!("Running stage {}...", cli_args.start_stage);
    }
    else {
        println!("Running from stage {} to stage {}...", cli_args.start_stage, end_stage);
    }

    let mut targets = Targets{
        layout_target: None,
        mesh_target: None,
        sim_target: None,
        matching_target: None,
        shared_args: cli_args.shared_args,
    };

    for stage in args::RunStage::iter() {
        if stage.stage_num() < cli_args.start_stage.stage_num() {
            continue;
        }
        if stage.stage_num() > end_stage.stage_num() {
            break;
        }

        match stage {
            args::RunStage::Layout => {
                if let Some(layout_cfg) = &cli_args.layout_cfg {
                    println!("Loading layout config file: {}", layout_cfg);
                    targets.layout_target = Some(layout::LayoutTarget::from_cfg(layout_cfg)?);
                }
                else {
                    return crate::err_string("Layout config file not specified".to_string());
                }
            },
            args::RunStage::Mesh => {
                if let Some(mesh_cfg) = &cli_args.mesh_cfg {
                    println!("Loading mesh config file: {}", mesh_cfg);
                    return crate::err_string("Mesh config not yet implemented!!!".to_string());
                }
                else {
                    return crate::err_string("Mesh config file not specified".to_string());
                }
            },
            args::RunStage::Sim => {
                if let Some(sim_cfg) = &cli_args.sim_cfg {
                    println!("Loading simulation config file: {}", sim_cfg);
                    return crate::err_string("Simulation config not yet implemented!!!".to_string());
                }
                else {
                    return crate::err_string("Simulation config file not specified".to_string());
                }
            },
            args::RunStage::Match => {
                if let Some(matching_cfg) = &cli_args.matching_cfg {
                    println!("Loading matching config file: {}", matching_cfg);
                    return crate::err_string("Matching config not yet implemented!!!".to_string());
                }
                else {
                    return crate::err_string("Matching config file not specified".to_string());
                }
            },
        }
    }

    Ok(targets)
}

/// [Stage 2.] TODO UNFINISHED FUNCTION
/// Run the process on the targets (layout, matching, or both).
/// Returns a `Result` with `()` or an `Err`.
#[allow(unused_variables)]
pub fn run_process(targets: Targets) -> crate::Result<()> {

    // 2.1 Run the layout process
    if let Some(layout_target) = targets.layout_target {
        println!("#################");
        println!("Running layout...");
        println!("#################");
        let layout_out = layout::do_layout(&layout_target)?;
    };

    // 2.2 Run the mesh process
    if let Some(mesh_target) = targets.mesh_target {
        println!("################");
        println!("Running mesh...");
        println!("################");
        return crate::err_string("Meshing not yet implemented!!!".to_string());
    }

    // 2.3 Run the simulation process
    if let Some(sim_target) = targets.sim_target {
        println!("####################");
        println!("Running simulation...");
        println!("####################");
        return crate::err_string("Simulation not yet implemented!!!".to_string());
    }

    // 2.4 Run the matching process
    if let Some(matching_target) = targets.matching_target {
        println!("##################");
        println!("Running matching...");
        println!("##################");
        return crate::err_string("Matching not yet implemented!!!".to_string());
    }

    Ok(())
}

/// Top-level tests
#[cfg(test)]
mod tests {
    
    // TODO: Add tests
}
