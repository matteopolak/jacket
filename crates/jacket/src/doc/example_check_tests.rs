//! The Node runner, against a stand-in browser bundle.
use std::process::Command;

use test_support::TempDir;

use super::*;

/// A browser bundle whose compiler spins for a few seconds on `spin` and fails on `fail`.
fn runtime() -> TempDir {
    let version = Command::new("node")
        .arg("--version")
        .output()
        .expect("these tests run the Node runner: install Node");

    assert!(version.status.success(), "`node --version` failed");
    let dir = TempDir::new("doc-example-runtime");

    std::fs::write(dir.join("jai_wasm.wasm"), b"fixture").unwrap();
    std::fs::write(
        dir.join("engine.mjs"),
        "export async function createEngine() {
  return { play(files) {
    const text = files['main.jai'];
    if (text === 'spin') {
      const end = Date.now() + 3000;
      while (Date.now() < end) {}
      return { exitCode: 0 };
    }
    if (text === 'fail') return { exitCode: 1, stderr: 'it failed' };
    return { exitCode: 0 };
  } };
}",
    )
    .unwrap();
    dir
}

fn program(label: &str, text: &str, timeout: Option<u64>) -> Program {
    Program {
        label: label.into(),
        text: text.into(),
        timeout,
    }
}

fn run_in(runtime: &TempDir, programs: &[Program]) -> Vec<Failure> {
    run(
        &Engine::Browser {
            node: "node".into(),
            runtime,
        },
        programs,
    )
    .unwrap()
}

#[test]
fn an_example_in_the_browser_is_stopped_at_its_own_timeout() {
    let runtime = runtime();
    let programs = [
        program("spins", "spin", Some(1)),
        program("after", "fail", Some(30)),
    ];
    let failures = run_in(&runtime, &programs);
    let labels: Vec<_> = failures.iter().map(|f| f.label.as_str()).collect();

    assert_eq!(labels, ["spins", "after"]);
    assert_eq!(failures[0].message, "Stopped after 1s.");
    assert_eq!(failures[1].message, "it failed");
}

#[test]
fn an_example_without_a_timeout_gets_the_default() {
    let runtime = runtime();

    assert!(run_in(&runtime, &[program("slow", "spin", None)]).is_empty());
}

#[cfg(unix)]
#[test]
fn a_shard_that_hangs_outside_its_programs_is_stopped() {
    use std::os::unix::fs::PermissionsExt;

    let runtime = runtime();
    let work = TempDir::new("doc-example-work");
    // A `node` that never answers: the bound is the programs' limits and one second.
    let node = work.join("node");

    std::fs::write(&node, "#!/bin/sh\nexec sleep 30\n").unwrap();
    std::fs::set_permissions(&node, std::fs::Permissions::from_mode(0o755)).unwrap();
    let error = browser(
        &node,
        &runtime,
        &[program("never", "ok", Some(1))],
        &work,
        Duration::from_secs(1),
    );

    assert_eq!(
        error.err().as_deref(),
        Some("the browser compiler did not finish within 2 seconds")
    );
}

#[test]
fn a_shard_may_take_its_programs_limits_and_the_startup() {
    let programs = [program("a", "", Some(5)), program("b", "", None)];

    assert_eq!(
        shard_limit(programs.iter(), Duration::from_secs(10)),
        Duration::from_secs(10 + 5) + DEFAULT_TIMEOUT
    );
}
