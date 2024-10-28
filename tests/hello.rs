use std::process::Command;

#[test]
fn interpreter_hello() {
    let output = Command::new("cargo")
        .args(["run", "--", "interpret", "hello.bf"])
        .output()
        .expect("Failed to execute test process");

    assert_eq!(output.stdout, b"Hello World!\n")
}

#[test]
fn compiler_hello() {
    let output = Command::new("cargo")
        .args(["run", "--", "compile", "hello.bf"])
        .output()
        .expect("Failed to execute test process");

    assert_eq!(output.stdout, b"Hello World!\n")
}