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
    if (text === 'needs util') {
      const util = files['modules/Util/module.jai'];
      return util === 'answer :: 42;\\n' && !('modules/Util/notes.txt' in files)
        ? { exitCode: 0 }
        : { exitCode: 1, stderr: 'Util was not supplied' };
    }
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
            modules: None,
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
fn the_modules_of_a_library_are_supplied_to_the_browser_compiler() {
    let runtime = runtime();
    let library = TempDir::new("doc-example-library");

    std::fs::create_dir_all(library.join("Util")).unwrap();
    std::fs::create_dir_all(library.join(".hidden")).unwrap();
    std::fs::write(library.join("Util/module.jai"), "answer :: 42;\n").unwrap();
    std::fs::write(library.join("Util/notes.txt"), "not source").unwrap();
    std::fs::write(library.join(".hidden/module.jai"), "skipped :: 0;\n").unwrap();
    let programs = [program("uses Util", "needs util", None)];
    let run_with = |modules| {
        run(
            &Engine::Browser {
                node: "node".into(),
                runtime: &runtime,
                modules,
            },
            &programs,
        )
        .unwrap()
    };

    assert!(run_with(Some(&library)).is_empty());
    assert_eq!(run_with(None)[0].message, "Util was not supplied");
}

#[test]
fn module_files_are_the_jai_sources_below_a_library_without_hidden_folders() {
    let library = TempDir::new("doc-example-module-files");

    std::fs::create_dir_all(library.join("A/internal")).unwrap();
    std::fs::create_dir_all(library.join(".git")).unwrap();
    std::fs::write(library.join("A/module.jai"), "a :: 1;").unwrap();
    std::fs::write(library.join("A/internal/x.jai"), "x :: 1;").unwrap();
    std::fs::write(library.join("A/readme.md"), "text").unwrap();
    std::fs::write(library.join(".git/y.jai"), "y :: 1;").unwrap();
    let files = module_files(&library).unwrap();

    assert_eq!(
        files.keys().map(String::as_str).collect::<Vec<_>>(),
        ["modules/A/internal/x.jai", "modules/A/module.jai"]
    );
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
        None,
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
