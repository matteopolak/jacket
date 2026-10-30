//! `jacket test --list` names the `@compile_time` tests without running them: an editor lists
//! on every refresh, and the compiler would otherwise run each body while it builds.
use std::path::{Path, PathBuf};
use std::process::Command;
use test_support::TempDir;

fn target_dir() -> PathBuf {
    Path::new(env!("CARGO_BIN_EXE_jacket"))
        .parent()
        .expect("jacket lives in a directory")
        .to_path_buf()
}

fn stdlib() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../stdlib")
}

/// A scratch directory of its own, so tests do not share files.
fn scratch(name: &str) -> TempDir {
    let dir = TempDir::new(&format!("jacket-list-{name}"));
    std::fs::create_dir_all(&dir).expect("create the scratch directory");
    dir
}

/// `jacket` and `jaic` side by side. `None` when `jaic` has not been built.
fn archive(root: &Path) -> Option<PathBuf> {
    let suffix = std::env::consts::EXE_SUFFIX;
    let jaic = target_dir().join(format!("jaic{suffix}"));
    if !jaic.is_file() {
        eprintln!("skipped: build `jaic` first (`cargo build -p jaic-cli`)");
        return None;
    }
    let bin = root.join("archive");
    std::fs::create_dir_all(&bin).expect("create the archive directory");
    std::fs::copy(&jaic, bin.join(format!("jaic{suffix}"))).expect("copy jaic");
    let jacket = bin.join(format!("jacket{suffix}"));
    std::fs::copy(env!("CARGO_BIN_EXE_jacket"), &jacket).expect("copy jacket");
    Some(jacket)
}

fn jacket(exe: &Path, dir: &Path, args: &[&str]) -> std::process::Output {
    Command::new(exe)
        .args(args)
        .current_dir(dir)
        .env("JAIC_STDLIB", stdlib())
        .output()
        .expect("start jacket")
}

const TESTS: &str = r#"#import "Basic";
#import "Extensions/Testing";

// Running this in the compiler ends the build.
ends_the_compile :: () {
    print("compile_time_marker\n");
    exit(7);
} @test @compile_time

fails_in_the_compiler :: () {
    expect(false, "compile_time_failure");
} @test @compile_time

plain :: () {
    expect(true);
} @test
"#;

#[test]
fn list_does_not_run_compile_time_tests() {
    let root = scratch("ct");
    let Some(exe) = archive(&root) else {
        return;
    };
    let project = root.join("project");
    std::fs::create_dir_all(project.join("tests")).expect("create tests");
    std::fs::write(project.join("tests/ct.jai"), TESTS).expect("write the tests");

    let args = ["test", "--list"];
    let out = jacket(&exe, &project, &args);
    let stdout = String::from_utf8_lossy(&out.stdout);
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(out.status.success(), "{stdout}{stderr}");
    assert_eq!(
        stdout.lines().collect::<Vec<_>>(),
        [
            "ends_the_compile: test",
            "fails_in_the_compiler: test",
            "plain: test"
        ],
        "{stderr}"
    );
}
