fn main() {
    let targets = comrade::parse_cli_args();

    comrade::do_layout();
    comrade::do_matching();
}
