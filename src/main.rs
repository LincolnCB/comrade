fn main() {

    // 1. Parse commandline arguments to get the targets
    let targets = match comrade::handle_cli_args(comrade::args::parse_cli_args()) {
        Ok(targets) => targets,
        Err(error) => {
            println!("Commandline Argument Error: {}", error);
            return;
        },
    };

    // 2. Run the process on the targets (layout, matching, or both)
    if let Err(error) = comrade::run_process(targets) {
        println!("Process Error: {}", error);
        return;
    };
}
