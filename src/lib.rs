pub mod layout;
pub mod matching;
pub mod args;

/// Targets struct.
/// This struct contains the layout and matching targets to do.
pub struct Targets{
    pub layout_target: Option<layout::LayoutStyle>,
    pub matching_target: Option<u32>, // TODO THIS IS A DUMMY
    pub shared_args: args::SharedArgs,
}

/// [Stage 1.] TODO UNFINISHED FUNCTION
/// Parse the command line arguments for the comrade binary.
/// Uses the `clap` crate.
/// Returns a `Result` with a `comrade::RunArgs` struct or an `Err`.
pub fn handle_cli_args(cli_args : args::ComradeCli) -> Result<Targets, String>{

    // 1.1 Handle the different subcommands
    match cli_args.sub_command {
        args::RunStage::Layout(layout_cli) => { // Layout command
            println!("Layout only");
            println!("{:?}", layout_cli);

            let (layout_style, shared_args) = layout_cli.reconstruct()?;
            Ok(Targets{
                layout_target: Some(layout_style),
                matching_target: None,
                shared_args,
            })
            
        },
        args::RunStage::Matching(matching_args) => { // Matching command
            println!("Matching only");
            println!("{:?}", matching_args);

            // Parse matching style

            Err("Matching not implemented yet".to_string())
        },
        args::RunStage::Full(full_args) => { // Full command
            println!("Full process");
            println!("{:?}", full_args);

            // Parse both styles

            Err("Full process not implemented yet".to_string())
        },
    }
}

/// [Stage 2.] TODO UNFINISHED FUNCTION
/// Run the process on the targets (layout, matching, or both).
/// Returns a `Result` with `()` or an `Err`.
pub fn run_process(targets: Targets) -> Result<(), String> {

    // 2.1 Run the layout process
    if let Some(layout_style) = targets.layout_target {
        println!("#################");
        println!("Running layout...");
        println!("#################");
        println!("Layout style: {}", get_style_name(&layout_style));
        let layout_out = layout::do_layout(&layout_style, &targets.shared_args)?;
        layout::mesh_layout(&layout_out, &targets.shared_args)?;
    };

    // 2.2 Run MARIE between the two processes
    // TODO: Figure out MARIE interface

    // 2.3 Run the matching process
    if let Some(matching_network) = targets.matching_target {
        println!("###################");
        println!("Running matching...");
        println!("###################");
        let matching_out = matching::do_matching()?;
        matching::save_matching(&matching_out, &targets.shared_args)?;
    }

    Ok(())
}

/// Get the name of the layout style.
pub fn get_style_name(layout_style: &layout::LayoutStyle) -> String {
    layout::IsStyle::get_style_name(layout_style)
}

/// Top-level tests
#[cfg(test)]
mod tests {
    use super::*;
    use crate::layout::IsStyle;

    /// Test the `handle_cli_args` function for `layout` command.
    #[test]
    fn handle_layout_args() {
        let arguments = args::parse_cli_from(&["comrade", "layout", "-i", "input.stl", "-o", "output", "-l", "400", "iterative-circle", "-c", "8"]);
        let targets = handle_cli_args(arguments).unwrap();
        assert_eq!(targets.shared_args.input_path, "input.stl");
        assert_eq!(targets.shared_args.output_name, "output");
        assert_eq!(targets.shared_args.larmor_mhz, 400.0);
        assert_eq!(targets.layout_target.unwrap().get_style_name(), "Iterative Circle");
    }
}
