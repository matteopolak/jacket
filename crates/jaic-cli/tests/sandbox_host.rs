//! Programs run on the sandbox host (`-interp -os wasm`), the engine the browser playground uses.
use std::path::Path;
use std::process::Command;
use test_support::TempDir;

const JAIC: &str = env!("CARGO_BIN_EXE_jaic");

/// The sandbox (`-os wasm`, the browser's engine) runs every thread on one host thread. A wait
/// nothing can end any more is reported as a deadlock instead of hanging, whether the thread
/// that could have ended it has returned or is blocked itself, and timed waits and sleeps move
/// the virtual clock: a short wait times out, a longer one sees the signal sent after a sleep.
#[test]
fn sandbox_threads_wake_waiters_and_report_deadlocks() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/compiler/semantics/sandbox-threads");
    for name in ["returned.jai", "blocked.jai", "timed.jai"] {
        let path = dir.join(name);
        let output = Command::new(JAIC)
            .arg("-interp")
            .arg(&path)
            .args(["-os", "wasm"])
            .output()
            .unwrap();
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        if name == "timed.jai" {
            assert!(output.status.success(), "{name}: {stderr}");
            assert_eq!(stdout, "TIMEOUT SUCCESS\n", "{name}");
        } else {
            assert!(!output.status.success(), "{name}: {stdout}");
            assert!(
                stderr.contains("deadlock: every thread is blocked"),
                "{name}: {stderr}"
            );
            assert!(stdout.is_empty(), "{name}: {stdout}");
        }
    }
}

/// Basic's clock, calendar, sleep and exit exist on the sandbox host the browser runs, where
/// sleeping moves the virtual clock.
#[test]
fn sandbox_has_basic_time_sleep_and_exit() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/compiler/semantics/sandbox-time.jai");
    let output = Command::new(JAIC)
        .arg("-interp")
        .arg(&path)
        .args(["-os", "wasm"])
        .output()
        .unwrap();
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        "true true true\n29 February 2024, 12:00:00\n",
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(output.status.code(), Some(3));
}

/// A wasm program run with `-interp` (in the sandbox host) gets its arguments too.
#[test]
fn wasm_interp_passes_program_arguments() {
    let dir = TempDir::new("cli-wasm-run-args");
    let source = dir.join("args.jai");
    std::fs::write(
        &source,
        "#import \"Basic\";\n\
         main :: () {\n\
         \x20   args := get_command_line_arguments();\n\
         \x20   for args if it_index > 0 print(\"[%]\", it);\n\
         \x20   print(\"%\\n\", args.count);\n\
         }\n",
    )
    .unwrap();
    let output = Command::new(JAIC)
        .args(["-interp", "-os", "wasm"])
        .arg(&source)
        .args(["--", "a", "b c"])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&output.stdout), "[a][b c]3\n");
}

/// A listener named by `JAIC_TCPLISTEN` becomes descriptor 3 of an `-interp -os wasm` run, and
/// the program echoes a message over the connection it accepts the way a WASI program does with
/// `sock_accept`, `sock_recv`, `sock_send` and `sock_shutdown`.
#[test]
fn sandbox_serves_a_preopened_listener() {
    use std::io::{Read, Write};
    use std::net::{Shutdown, TcpListener, TcpStream};

    let port = TcpListener::bind("127.0.0.1:0")
        .unwrap()
        .local_addr()
        .unwrap()
        .port();
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/compiler/semantics/sandbox-sockets.jai");
    let child = Command::new(JAIC)
        .arg("-interp")
        .arg(&path)
        .args(["-os", "wasm"])
        .env("JAIC_TCPLISTEN", format!("127.0.0.1:{port}"))
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .unwrap();
    // The listener exists before the program is compiled, so a connection waits for `accept`.
    let mut stream = None;
    for _ in 0..200 {
        match TcpStream::connect(("127.0.0.1", port)) {
            Ok(connected) => {
                stream = Some(connected);
                break;
            }
            Err(_) => std::thread::sleep(std::time::Duration::from_millis(50)),
        }
    }
    let mut stream = stream.expect("the listener never came up");
    let _ = stream.write_all(b"copper");
    let _ = stream.shutdown(Shutdown::Write);
    let mut reply = String::new();
    let read = stream.read_to_string(&mut reply);
    let output = child.wait_with_output().unwrap();
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(output.status.success(), "{stderr}");
    read.unwrap();
    assert_eq!(reply, "echo: copper");
    assert_eq!(String::from_utf8_lossy(&output.stdout), "ok\n", "{stderr}");
}
