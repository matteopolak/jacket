//! A compile-fail test with no `.stderr` yet gets one on the first run, and says so; under `CI`
//! the missing file is a failure.
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use test_support::TempDir;

/// The directory holding `jaic` beside `jacket`; `None` when `jaic` has not been built.
fn jaic_dir() -> Option<PathBuf> {
    let dir = Path::new(env!("CARGO_BIN_EXE_jacket"))
        .parent()?
        .to_path_buf();
    dir.join("jaic").is_file().then_some(dir)
}

fn project() -> TempDir {
    let project = TempDir::new("jacket-new-golden");
    let tests = project.join("tests").join("compile_fail");
    std::fs::create_dir_all(&tests).unwrap();
    std::fs::write(project.join("jacket.toml"), "[project]\n").unwrap();
    std::fs::write(tests.join("bad.jai"), "main :: () { x: int = \"s\"; }\n").unwrap();
    project
}

fn run(project: &Path, ci: bool) -> Output {
    let mut command = Command::new(env!("CARGO_BIN_EXE_jacket"));
    command
        .args(["test", "--no-doc"])
        .current_dir(project)
        .env(
            "JAIC_STDLIB",
            Path::new(env!("CARGO_MANIFEST_DIR")).join("../../stdlib"),
        )
        .env_remove("CI");
    if ci {
        command.env("CI", "true");
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
fn a_missing_compile_fail_stderr_is_written_on_the_first_run() {
    if jaic_dir().is_none() {
        eprintln!("skipped: build `jaic` first (`cargo build -p jaic-cli`)");
        return;
    }
    let project = project();
    let expected = project.join("tests/compile_fail/bad.stderr");

    let first = run(&project, false);
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

    // The next run compares against what the first wrote.
    let second = run(&project, false);
    assert!(second.status.success(), "{}", text(&second));
    assert!(!text(&second).contains("CREATED"));
}

#[test]
fn under_ci_a_missing_stderr_still_fails_and_is_not_written() {
    if jaic_dir().is_none() {
        eprintln!("skipped: build `jaic` first (`cargo build -p jaic-cli`)");
        return;
    }
    let project = project();
    let out = run(&project, true);
    let shown = text(&out);
    assert!(!out.status.success(), "{shown}");
    assert!(shown.contains("bad.stderr is missing"), "{shown}");
    assert!(!project.join("tests/compile_fail/bad.stderr").exists());
}
