use assert_cmd::Command;

#[test]
fn check_cargo_test() {
    assert_eq!(2 + 2, 4);
}

#[test]
fn test_subcommand_options(){
    let mut cmd = Command::cargo_bin("comrade").unwrap();

    let expected_stderr = concat!(
        "Constrained Optimization for Magnetic Resonance Array Design tool\n",
        "\n",
        "Usage: comrade <COMMAND>\n",
        "\n",
        "Commands:\n",
        "  layout  Run the layout process only, outputting a layout file and optional mesh\n",
        "  match   Run the matching process only, outputting a matching file\n",
        "  full    Run the full process, outputting a layout file, optional mesh, and matching file\n",
        "  help    Print this message or the help of the given subcommand(s)\n",
        "\n",
        "Options:\n",
        "  -h, --help  Print help\n",
    );
    cmd.assert().failure().stderr(expected_stderr);
}
