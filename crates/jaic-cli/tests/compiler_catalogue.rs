//! Original public compiler capability diagnostics retained during fixture migration.
use std::{path::PathBuf, process::Command};

#[test]
fn quoted_insert_executes_its_local_assertions_and_rejects_local_duplicates() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let output = Command::new(env!("CARGO_BIN_EXE_jaic"))
        .env("JAIC_STDLIB", root.join("stdlib"))
        .arg(root.join("tests/compiler-semantics/insert-same-scope-declarations.jai"))
        .arg("-interp")
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let output = Command::new(env!("CARGO_BIN_EXE_jaic"))
        .env("JAIC_STDLIB", root.join("stdlib"))
        .arg(root.join("tests/compiler-semantics/insert-same-scope-duplicate-rejected.jai"))
        .arg("-check")
        .output()
        .unwrap();
    assert!(!output.status.success());
    assert!(
        String::from_utf8_lossy(&output.stderr)
            .contains("`first` is already declared in this scope")
    );
}

#[test]
fn custom_iteration_remove_and_operator_values_preserve_semantics() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    for name in ["custom-for-array-remove", "operator-name-values"] {
        let output = Command::new(env!("CARGO_BIN_EXE_jaic"))
            .env("JAIC_STDLIB", root.join("stdlib"))
            .arg(root.join(format!("tests/compiler-semantics/{name}.jai")))
            .arg("-interp")
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{name}: {}{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }
    let output = Command::new(env!("CARGO_BIN_EXE_jaic"))
        .env("JAIC_STDLIB", root.join("stdlib"))
        .arg(root.join("tests/compiler-semantics/custom-for-range-remove-rejected.jai"))
        .arg("-check")
        .output()
        .unwrap();
    assert!(!output.status.success());
    assert!(
        String::from_utf8_lossy(&output.stderr)
            .contains("remove is only valid inside a for loop over an array")
    );
}

#[test]
fn a_qualified_namespace_does_not_implicitly_import_operators() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("operator-namespace-visibility");
    std::fs::create_dir_all(&dir).unwrap();
    let source = dir.join("main.jai");
    std::fs::write(
        &source,
        "Probe :: #import \"Operator_Probe\"; main :: () { value: Probe.Scalar; result := -value; }\n",
    )
    .unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_jaic"))
        .env("JAIC_STDLIB", root.join("stdlib"))
        .arg(source)
        .arg("-check")
        .arg("-I")
        .arg(root.join("tests/compiler-semantics/modules"))
        .output()
        .unwrap();
    assert!(!output.status.success());
    let error = String::from_utf8_lossy(&output.stderr);
    assert!(error.contains("cannot negate Scalar"), "{error}");
}

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
        "code-roundtrip-narrowing-rejected",
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
fn source_reconstruction_executes_original_semantics_and_retains_rejections() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    for name in [
        "statements",
        "control",
        "types",
        "attributes",
        "imports",
        "literals",
        "assembly",
        "aliases",
        "enum-members",
        "array-flags",
    ] {
        let output = Command::new(env!("CARGO_BIN_EXE_jaic"))
            .env("JAIC_STDLIB", root.join("stdlib"))
            .arg(root.join(format!(
                "tests/compiler-semantics/code-roundtrip-{name}.jai"
            )))
            .arg("-interp")
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{name}: {}{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }
    let output = Command::new(env!("CARGO_BIN_EXE_jaic"))
        .env("JAIC_STDLIB", root.join("stdlib"))
        .arg(root.join("tests/compiler-semantics/code-roundtrip-module-map-rejected.jai"))
        .arg("-check")
        .output()
        .unwrap();
    assert!(!output.status.success());
    assert!(
        String::from_utf8_lossy(&output.stderr)
            .contains("using,map on modules is not supported by the reviewed contract")
    );
    let output = Command::new(env!("CARGO_BIN_EXE_jaic"))
        .env("JAIC_STDLIB", root.join("stdlib"))
        .arg(root.join("tests/compiler-semantics/code-modify-control.jai"))
        .arg("-check")
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn retained_member_and_real_library_bodies_are_modifiable() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    for path in [
        "tests/compiler-semantics/code-modify-members.jai",
        "tests/developer-tools/compiler-requests/profiler-member-modify.jai",
        "tests/developer-tools/compiler-requests/profiler-stdlib-modify.jai",
        "tests/developer-tools/compiler-requests/profiler-array-local-modify.jai",
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
