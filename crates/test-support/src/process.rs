//! Running child processes and checking what they did.

use std::borrow::Borrow;
use std::process::{Command, Output};

/// `output` rendered for a failure message: the status, then standard output and error.
#[must_use]
pub fn describe(output: &Output) -> String {
    format!(
        "status {:?}\n{}\n{}",
        output.status.code(),
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    )
}

/// Runs `command` to completion and returns its output, whatever the exit status. Panics only
/// when the process cannot be started.
pub fn output(command: &mut Command) -> Output {
    command
        .output()
        .unwrap_or_else(|error| panic!("cannot run {command:?}: {error}"))
}

/// Runs `command`, fails the test with its output unless it succeeds, and returns it.
pub fn succeeded(command: &mut Command) -> Output {
    let output = output(command);
    assert!(
        output.status.success(),
        "{command:?}: {}",
        describe(&output)
    );
    output
}

/// [`succeeded`] for a command whose output does not matter.
pub fn run(command: &mut Command) {
    succeeded(command);
}

/// [`succeeded`], returning standard output as text.
pub fn checked(command: &mut Command) -> String {
    String::from_utf8(succeeded(command).stdout).expect("standard output is UTF-8")
}

/// Runs `command` and requires it to succeed without printing anything.
pub fn success_silent(command: &mut Command) {
    let output = succeeded(command);
    assert!(
        output.stdout.is_empty(),
        "{command:?} printed {}",
        String::from_utf8_lossy(&output.stdout)
    );
}

/// Requires `output` to be a success whose standard output is exactly `expected`.
pub fn success_with_stdout(output: impl Borrow<Output>, expected: &str) {
    let output = output.borrow();
    assert!(output.status.success(), "{}", describe(output));
    assert_eq!(String::from_utf8_lossy(&output.stdout), expected);
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;

    fn shell(script: &str) -> Command {
        let mut command = Command::new("sh");
        command.args(["-c", script]);
        command
    }

    #[test]
    #[cfg(unix)]
    fn checked_returns_stdout() {
        assert_eq!(checked(&mut shell("printf hi")), "hi");
    }

    #[test]
    #[cfg(unix)]
    #[should_panic(expected = "boom")]
    fn failures_show_the_output() {
        run(&mut shell("echo boom >&2; exit 3"));
    }

    #[test]
    #[cfg(unix)]
    #[should_panic(expected = "printed")]
    fn silent_commands_must_print_nothing() {
        success_silent(&mut shell("echo noise"));
    }
}
