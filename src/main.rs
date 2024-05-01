fn main() {

    let cli = comrade::args::parse_cli_args();

    match cli.subcommand {
        comrade::args::SubCommand::Example(example_args) => {
            match comrade::example::display_config(example_args){
                Ok(_) => {},
                Err(err) => println!("{}", err),
            }
            return;
        },
        comrade::args::SubCommand::Run(run_args) => {
            let targets = match comrade::build_targets(run_args) {
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
        },
    }
}
