//! Original public compiler capability diagnostics retained during fixture migration.
use std::{path::PathBuf, process::Command};

#[test]
fn called_library_body_interception_and_filters_complete_successfully() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    for path in [
        "tests/producer-requests/profiler-stdlib-bodies.jai",
        "tests/compiler-semantics/interception-canonical-bodies.jai",
    ] {
        let output = Command::new(env!("CARGO_BIN_EXE_jaic"))
            .env("JAIC_STDLIB", root.join("stdlib"))
            .arg(root.join(path))
            .arg("-check")
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{path}: {}{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }
}

#[test]
fn decimal_required_precision_rejects_implicit_narrowing() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    for name in [
        "decimal-implicit-narrowing-rejected",
        "decimal-arithmetic-narrowing-rejected",
    ] {
        let output = Command::new(env!("CARGO_BIN_EXE_jaic"))
            .env("JAIC_STDLIB", root.join("stdlib"))
            .arg(root.join(format!("tests/compiler-semantics/{name}.jai")))
            .arg("-check")
            .output()
            .unwrap();
        assert!(!output.status.success());
        assert!(String::from_utf8_lossy(&output.stderr).contains("requires float64 precision"));
    }
}

#[test]
fn unavailable_debugger_is_a_required_diagnostic() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let output = Command::new(env!("CARGO_BIN_EXE_jaic"))
        .env("JAIC_STDLIB", root.join("stdlib"))
        .arg(root.join("tests/compiler-semantics/compiler-debugger-unavailable.jai"))
        .arg("-check")
        .output()
        .unwrap();
    assert!(!output.status.success());
    assert!(
        String::from_utf8_lossy(&output.stderr)
            .contains("developer_debug requires compiler debugger support")
    );
}
