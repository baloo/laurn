use std::env;
use std::path::PathBuf;
use std::process::{Command, Stdio};

#[test]
fn test_basic_environment() {
    let laurn = PathBuf::from(env!("CARGO_BIN_EXE_laurn"));
    let test_data = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/basic/laurn.nix");

    let output = Command::new(laurn)
        .arg("run")
        .arg("-p")
        .arg(test_data)
        .arg("echo I run in a container")
        .stdin(Stdio::null())
        .stderr(Stdio::inherit())
        .output()
        .expect("unable to run laurn")
        .stdout;

    assert_eq!(output, b"I run in a container\n");
}
