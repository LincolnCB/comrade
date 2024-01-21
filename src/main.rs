fn main() {

    // 1. Parse commandline arguments to get the targets
    let targets = match comrade::handle_cli_args(comrade::args::parse_cli_args()) {
        Ok(targets) => targets,
        Err(err) => {
            println!("CLI ERROR!");
            match err {
                comrade::ComradeError::IOError(err) => {
                    println!("IO Error: {}", err);
                },
                comrade::ComradeError::Default(err) => {
                    println!("{}", err);
                },
                _ => {
                    println!("UNHANDLED CLI ERROR");
                },
            }
            return;
        },
    };

    // 2. Run the process on the targets (layout, matching, or both)
    if let Err(err) = comrade::run_process(targets) {
        println!("PROCESS ERROR!");
        match err {
            comrade::ComradeError::IOError(err) => {
                println!("IO Error: {}", err);
            },
            comrade::ComradeError::Default(err) => {
                println!("{}", err);
            },
            _ => {
                println!("UNHANDLED PROCESS ERROR");
            },
        }
        return;
    };
}
