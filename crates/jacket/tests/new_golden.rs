//! A compile-fail test with no `.stderr` fails, whatever `CI` says; with `--create-missing` it gets
//! one from what the compiler printed, and the run says so.
use std::path::Path;
use std::process::{Command, Output};

use test_support::{TempDir, toolchain};

fn project() -> TempDir {
    let project = TempDir::new("jacket-new-golden");
    let tests = project.join("tests").join("compile_fail");

    std::fs::create_dir_all(&tests).unwrap();
    std::fs::write(project.join("jacket.toml"), "[project]\n").unwrap();
    std::fs::write(tests.join("bad.jai"), "main :: () { x: int = \"s\"; }\n").unwrap();
    project
}

fn run(project: &Path, create_missing: bool) -> Output {
    let mut command = Command::new(env!("CARGO_BIN_EXE_jacket"));

    command
        .args(["test", "--no-doc"])
        .current_dir(project)
        .env(
            "JAIC_STDLIB",
            Path::new(env!("CARGO_MANIFEST_DIR")).join("../../stdlib"),
        )
        .env_remove("CI");

    if create_missing {
        command.arg("--create-missing");
    }

    command.output().expect("start jacket")
}

fn text(out: &Output) -> String {
    format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    )
}

#[test]
fn create_missing_writes_a_missing_compile_fail_stderr() {
    if toolchain::jaic_beside(Path::new(env!("CARGO_BIN_EXE_jacket"))).is_none() {
        return;
    }

    let project = project();
    let expected = project.join("tests/compile_fail/bad.stderr");

    let first = run(&project, true);
    let shown = text(&first);

    assert!(first.status.success(), "{shown}");
    assert!(shown.contains("CREATED"), "{shown}");
    assert!(shown.contains("bad.stderr"), "{shown}");
    assert!(
        std::fs::read_to_string(&expected)
            .unwrap()
            .contains("error"),
        "the written file holds the diagnostics"
    );

    // The next run, without the flag, compares against what the first wrote.
    let second = run(&project, false);

    assert!(second.status.success(), "{}", text(&second));
    assert!(!text(&second).contains("CREATED"));
}

#[test]
fn a_missing_stderr_fails_and_is_not_written_without_the_flag() {
    if toolchain::jaic_beside(Path::new(env!("CARGO_BIN_EXE_jacket"))).is_none() {
        return;
    }

    let project = project();
    let out = run(&project, false);
    let shown = text(&out);

    assert!(!out.status.success(), "{shown}");
    assert!(shown.contains("bad.stderr is missing"), "{shown}");
    assert!(!project.join("tests/compile_fail/bad.stderr").exists());
}
