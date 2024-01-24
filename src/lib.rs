pub mod layout;
pub mod mesh;
pub mod sim;
pub mod matching;
pub mod args;
mod crate_errors;

use strum::IntoEnumIterator;

pub use crate_errors::{
    ComradeError,
    ComradeResult,
    err_str,
};

/// Targets struct.
/// This struct contains the layout and matching targets to do.
pub struct Targets{
    pub layout_target: Option<layout::LayoutTarget>,
    pub mesh_target: Option<()>, // TODO THIS IS A DUMMY
    pub sim_target: Option<()>, // TODO THIS IS A DUMMY
    pub matching_target: Option<()>, // TODO THIS IS A DUMMY
    pub shared_args: args::SharedArgs,
}

/// [Stage 1.]
/// Parse the command line arguments for the comrade binary.
/// Uses the `clap` crate.
/// Expects to see a start stage and an optional end stage that must come after the start
/// For each stage to be run between them, checks for a required corresponding config file.
/// Returns a `ProcResult` with the `Targets` or an `Err`.
pub fn build_targets(cli_args : args::ComradeCli) -> ComradeResult<Targets>{
    let end_stage = if let Some(end_stage) = cli_args.end_stage {
        end_stage
    } else {
        cli_args.start_stage.clone()
    };

    if cli_args.start_stage.stage_num() > end_stage.stage_num() {
        args::err_str(&format!("Start stage ({}) is after end stage ({})", cli_args.start_stage, end_stage))?;
    }
    if cli_args.start_stage.stage_num() == end_stage.stage_num() {
        println!("Stage to run: {}...", cli_args.start_stage);
    }
    else {
        println!("Stages to run: {} through {} ...", cli_args.start_stage, end_stage);
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
        let is_last = stage.stage_num() == end_stage.stage_num();

        match stage {
            args::RunStage::Layout => {
                if let Some(layout_cfg) = &cli_args.layout_cfg {
                    println!("Loading layout config file: {}", layout_cfg);
                    targets.layout_target = Some(layout::LayoutTarget::from_cfg(
                        layout_cfg,
                        is_last
                    )?);
                }
                else {
                    args::err_str("Layout config file not specified")?;
                }
            },
            args::RunStage::Mesh => {
                if let Some(mesh_cfg) = &cli_args.mesh_cfg {
                    println!("Loading mesh config file: {}", mesh_cfg);
                    args::err_str("Mesh config not yet implemented!!!")?;
                }
                else {
                    args::err_str("Mesh config file not specified")?;
                }
            },
            args::RunStage::Sim => {
                if let Some(sim_cfg) = &cli_args.sim_cfg {
                    println!("Loading simulation config file: {}", sim_cfg);
                    args::err_str("Simulation config not yet implemented!!!")?;
                }
                else {
                    args::err_str("Simulation config file not specified")?;
                }
            },
            args::RunStage::Match => {
                if let Some(matching_cfg) = &cli_args.matching_cfg {
                    println!("Loading matching config file: {}", matching_cfg);
                    args::err_str("Matching config not yet implemented!!!")?;
                }
                else {
                    args::err_str("Matching config file not specified")?;
                }
            },
        }
    }

    Ok(targets)
}

/// [Stage 2.] TODO UNFINISHED FUNCTION
/// Run the process on the targets (layout, matching, or both).
/// Returns a `ProcResult` with `()` or an `Err`.
#[allow(unused_variables)]
pub fn run_process(targets: Targets) -> ComradeResult<()> {

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
        mesh::err_str("Meshing not yet implemented!!!")?;
    }

    // 2.3 Run the simulation process
    if let Some(sim_target) = targets.sim_target {
        println!("####################");
        println!("Running simulation...");
        println!("####################");
        sim::err_str("Simulation not yet implemented!!!")?;
    }

    // 2.4 Run the matching process
    if let Some(matching_target) = targets.matching_target {
        println!("##################");
        println!("Running matching...");
        println!("##################");
        matching::err_str("Matching not yet implemented!!!")?;
    }

    Ok(())
}

/// Top-level tests
#[cfg(test)]
mod tests {
    
    // TODO: Add tests
}
