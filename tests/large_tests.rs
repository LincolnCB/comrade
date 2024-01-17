use assert_cmd::Command;

#[test]
#[ignore]
fn do_layout() {
    let mut cmd = Command::cargo_bin("comrade").unwrap();

    cmd.arg("layout")
        .arg("iterative-circle")
        .arg("--input")
        .arg("tests/data/basic.stl")
        .arg("--larmor")
        .arg("127.8")
        .arg("--output")
        .arg("tests/data/layout_test")
        .arg("--count")
        .arg("4")
        .arg("--mesh")
        .assert()
        .success();
}
