fn main() {

    // 1. Parse commandline arguments to get the targets
    let targets = match comrade::build_targets(comrade::args::parse_cli_args()) {
        Ok(targets) => targets,
        Err(err) => {
            println!("{}", err);
            return;
        },
    };

    // 2. Run the process on the list of targets
    if let Err(err) = comrade::run_process(targets) {
        println!("{}", err);
        return;
    };
}
