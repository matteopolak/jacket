//! A project's own module named like a library one (`Windows`, `Objective_C`) replaces it for
//! every file that imports it, and the library's native code does not depend on it.
//!
//! Each fixture under `tests/module_replacement/` is a project whose `modules/` folder holds the
//! replacement. `-check -no_dce` type-checks the library code for the other OS as well, so the
//! Windows fixture runs on any host. The minimal replacements declare almost nothing: the library
//! internals only check when they use their own private declarations rather than the project's
//! module.
use std::path::Path;
use std::process::Command;

const JAIC: &str = env!("CARGO_BIN_EXE_jaic");

fn check(fixture: &str, os: &str, cpu: &str) {
    let main = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/module_replacement")
        .join(fixture)
        .join("main.jai");
    let output = Command::new(JAIC)
        .arg(&main)
        .args(["-check", "-no_dce", "-os", os, "-cpu", cpu])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{fixture} for {os}-{cpu}:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn a_project_windows_module_does_not_reach_library_internals() {
    for cpu in ["x64", "arm64"] {
        check("windows", "windows", cpu);
    }
}

#[test]
fn a_project_objective_c_module_does_not_reach_library_internals() {
    for cpu in ["x64", "arm64"] {
        check("objective_c_minimal", "macos", cpu);
    }
}

#[test]
fn window_creation_works_with_a_project_objective_c_module() {
    for cpu in ["x64", "arm64"] {
        check("objective_c_window", "macos", cpu);
    }
}
