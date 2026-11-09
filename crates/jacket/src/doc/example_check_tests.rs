//! The Node runner, against a stand-in browser bundle.
use super::*;
use test_support::TempDir;

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
