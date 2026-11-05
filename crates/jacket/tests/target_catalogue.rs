//! Selected-target discovery must match the cases that conditional compilation makes runnable.
#![cfg(unix)]
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::process::Command;
use test_support::TempDir;

fn run(project: &Path, listing: Option<bool>) -> Vec<serde_json::Value> {
    let mut command = Command::new(env!("CARGO_BIN_EXE_jacket"));
    command.args(["test", "--interp", "--no-doc", "--format", "json"]);
    if let Some(target) = listing {
        command.arg("--list");
        if target {
            command.arg("--list-target");
        }
    }
    let output = command
        .args(["--", "-os=wasm"])
        .current_dir(project)
        .env(
            "JAIC_STDLIB",
            Path::new(env!("CARGO_MANIFEST_DIR")).join("../../stdlib"),
        )
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect()
}

fn names(records: &[serde_json::Value], event: Option<&str>) -> BTreeSet<String> {
    records
        .iter()
        .filter(|record| {
            record["kind"] == "test"
                && match event {
                    Some(event) => record["event"] == event,
                    None => record["type"] == "list",
                }
        })
        .map(|record| record["name"].as_str().unwrap().to_owned())
        .collect()
}

fn project() -> TempDir {
    let compiler = PathBuf::from(env!("CARGO_BIN_EXE_jacket"))
        .parent()
        .unwrap()
        .join("jaic");
    assert!(
        compiler.is_file(),
        "build the paired debug compiler with cargo build -p jaic-cli"
    );
    let project = TempDir::new("target-catalogue");
    std::fs::create_dir_all(project.join("tests")).unwrap();
    std::fs::write(project.join("jacket.toml"), "[project]\n").unwrap();
    std::fs::write(
        project.join("tests/conditional.jai"),
        r#"
#import "Basic";
#if OS == .WASM {
    #load "../target-only.jai";
    wasm_only :: () { assert(target_value == 1); } @test
} else {
    host_only :: () {} @test
}
"#,
    )
    .unwrap();
    std::fs::write(project.join("target-only.jai"), "target_value :: 1;\n").unwrap();
    project
}

#[test]
fn target_catalogue_matches_actual_wasm_execution_without_changing_host_listing() {
    let project = project();
    let host = names(&run(&project, Some(false)), None);
    assert_eq!(host.len(), 1);
    assert!(host.iter().next().unwrap().ends_with("host_only"));
    let target = names(&run(&project, Some(true)), None);
    assert_eq!(target.len(), 1);
    assert!(target.iter().next().unwrap().ends_with("wasm_only"));
    for target in [false, true] {
        let mut command = Command::new(env!("CARGO_BIN_EXE_jacket"));
        command.args(["test", "--list", "--deps", "--interp"]);
        if target {
            command.arg("--list-target");
        }
        let output = command
            .args(["--", "-os=wasm"])
            .current_dir(&project)
            .env(
                "JAIC_STDLIB",
                Path::new(env!("CARGO_MANIFEST_DIR")).join("../../stdlib"),
            )
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert_eq!(
            String::from_utf8_lossy(&output.stdout).contains("target-only.jai"),
            target
        );
    }
    let executed = run(&project, None);
    assert_eq!(names(&executed, Some("passed")), target);
    assert!(names(&executed, Some("skipped")).is_empty());
}

#[test]
fn changed_selection_uses_the_actual_mode_target_catalogue() {
    let project = project();
    let names_file = project.join("selected.txt");
    let changed = serde_json::to_string(&[project.join("target-only.jai")]).unwrap();
    let output = Command::new("python3")
        .arg(Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tools/ci/select_tests.py"))
        .args(["--jacket", env!("CARGO_BIN_EXE_jacket"), "--out"])
        .arg(&names_file)
        .args(["--", "--interp", "--", "-os=wasm"])
        .current_dir(&project)
        .env("JACKET_TEST_CHANGED", changed)
        .env(
            "JAIC_STDLIB",
            Path::new(env!("CARGO_MANIFEST_DIR")).join("../../stdlib"),
        )
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let selected: BTreeSet<_> = std::fs::read_to_string(names_file)
        .unwrap()
        .lines()
        .map(str::to_owned)
        .collect();
    assert_eq!(selected, names(&run(&project, Some(true)), None));
    assert!(selected.iter().next().unwrap().ends_with("wasm_only"));
    let output = Command::new(env!("CARGO_BIN_EXE_jacket"))
        .args([
            "test",
            "--interp",
            "--no-doc",
            "--format",
            "json",
            "--changed",
        ])
        .arg(project.join("target-only.jai"))
        .args(["--", "-os=wasm"])
        .current_dir(&project)
        .env(
            "JAIC_STDLIB",
            Path::new(env!("CARGO_MANIFEST_DIR")).join("../../stdlib"),
        )
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let records: Vec<serde_json::Value> = String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect();
    assert_eq!(names(&records, Some("passed")), selected);
}
