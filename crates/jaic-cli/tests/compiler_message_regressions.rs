//! Regression programs that drive the `Compiler` module (messages, typed code, type information)
//! and `#run` results through the interpreter. They also run in the sweep; running them here keeps
//! the code behind the exported records covered by `cargo test`.
use std::{path::PathBuf, process::Command};

/// Interpret `tests/compiler/regressions/<name>.jai` and require a clean exit.
fn run_interpreted(name: &str) {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let output = Command::new(env!("CARGO_BIN_EXE_jaic"))
        .env("JAIC_STDLIB", root.join("stdlib"))
        .arg(root.join(format!("tests/compiler/regressions/{name}.jai")))
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

#[test]
fn message_and_record_exports_run_interpreted() {
    for name in [
        "compiler-performance-and-dump-messages",
        "compiler-make-procedure-live",
        "compiler-typechecked-messages",
        "compiler-typechecked-names",
        "compiler-typechecked-notes",
        "compiler-typechecked-polymorph-static-if",
        "compiler-typechecked-unused-and-global-flags",
        "compiler-import-control",
        "compiler-resolved-nodes",
        "compiler-get-code",
        "compiler-get-code-rerun",
        "compiler-header-own-types",
        "compiler-struct-location",
        "compiler-type-table-compile-time",
        "rules-compiler-module",
        "rules-compiler-records",
        "rules-workspaces",
    ] {
        run_interpreted(name);
    }
}

#[test]
fn type_information_and_code_export_programs_run_interpreted() {
    for name in [
        "rules-reflection-and-type-info",
        "rules-type-values-and-info",
        "poly-struct-type-names",
        "code-export-varargs-flag",
        "compiler-enum-external-type",
        "compiler-noted-module-procs",
        "compiler-reflection-pure",
        "compiler-module-placeholder-insert",
        "rules-code-values-and-insertion",
    ] {
        run_interpreted(name);
    }
}
