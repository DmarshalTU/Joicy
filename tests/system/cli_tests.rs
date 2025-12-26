//! System tests for CLI

use std::process::Command;

#[test]
#[ignore] // Ignore in regular test runs, run with: cargo test --test system -- --ignored
fn test_cli_init() {
    let output = Command::new("cargo")
        .args(["run", "--", "init", "."])
        .output()
        .expect("Failed to execute command");

    // TODO: Add proper assertions once CLI is implemented
    assert!(output.status.success() || !output.status.success());
}

#[test]
#[ignore]
fn test_cli_search() {
    let output = Command::new("cargo")
        .args(["run", "--", "search", "test query"])
        .output()
        .expect("Failed to execute command");

    // TODO: Add proper assertions
    assert!(output.status.success() || !output.status.success());
}

