//! Process helpers shared by the backend integration tests.
#![allow(dead_code)]
use std::process::Command;

/// Run `command` and fail the test with its output unless it succeeds.
pub fn run(command: &mut Command) {
    checked(command);
}

/// Run `command`, fail the test with its output unless it succeeds, and return its stdout.
pub fn checked(command: &mut Command) -> String {
    let output = command
        .output()
        .unwrap_or_else(|error| panic!("cannot run {command:?}: {error}"));
    assert!(
        output.status.success(),
        "{command:?}: status {:?}\n{}\n{}",
        output.status.code(),
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).unwrap()
}
