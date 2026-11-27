//! Running child processes and checking what they did.

use std::borrow::Borrow;
use std::io::Read;
use std::process::{Command, Output, Stdio};
use std::sync::mpsc;
use std::time::{Duration, Instant};

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

/// Runs `command` and returns its output, or `None` after killing it when it is still running
/// once `limit` has passed, or when a process it started still holds its output open a few seconds
/// after it exited. For tools that can hang on a machine that lacks something they need
/// (a debugger without permission to attach). Panics only when the process cannot be started.
pub fn output_within(command: &mut Command, limit: Duration) -> Option<Output> {
    let mut child = command
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap_or_else(|error| panic!("cannot run {command:?}: {error}"));
    fn drain(mut pipe: impl Read + Send + 'static) -> mpsc::Receiver<Vec<u8>> {
        let (send, receive) = mpsc::channel();
        std::thread::spawn(move || {
            let mut bytes = Vec::new();
            let _ = pipe.read_to_end(&mut bytes);
            let _ = send.send(bytes);
        });
        receive
    }
    let stdout = drain(child.stdout.take().expect("piped"));
    let stderr = drain(child.stderr.take().expect("piped"));
    let deadline = Instant::now() + limit;
    let status = loop {
        if let Some(status) = child.try_wait().expect("waiting for the child") {
            break status;
        }
        if Instant::now() >= deadline {
            let _ = child.kill();
            let _ = child.wait();
            return None;
        }
        std::thread::sleep(Duration::from_millis(20));
    };
    // A process the child started can hold the pipes open after the child is gone, so the readers
    // get a few seconds, not forever.
    let grace = Duration::from_secs(5);
    let stdout = stdout.recv_timeout(grace).ok()?;
    let stderr = stderr.recv_timeout(grace).ok()?;
    Some(Output {
        status,
        stdout,
        stderr,
    })
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
    fn a_command_that_outlives_its_limit_is_killed() {
        let started = Instant::now();
        assert!(output_within(&mut shell("sleep 30"), Duration::from_millis(200)).is_none());
        assert!(started.elapsed() < Duration::from_secs(10));
        // A grandchild that keeps the pipes open after the kill does not hold the call up.
        let started = Instant::now();
        let held = shell("sleep 30 & wait");
        assert!(output_within(&mut { held }, Duration::from_millis(200)).is_none());
        assert!(started.elapsed() < Duration::from_secs(10));
        let done = output_within(&mut shell("printf hi"), Duration::from_secs(30)).unwrap();
        assert_eq!(done.stdout, b"hi");
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
