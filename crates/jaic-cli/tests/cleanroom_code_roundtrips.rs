//! Public AST/printer roundtrips through real source-built tools.
use std::{
    path::PathBuf,
    process::{Command, Output},
};

const CASES: &[(&str, &str)] = &[
    ("stdlib-contract/compiler-using-records/main.jai", "ok\n"),
    (
        "fixtures/code-roundtrip/using-parameter-roundtrip.jai",
        "using parameters ok\n",
    ),
    (
        "fixtures/code-roundtrip/named-while-roundtrip.jai",
        "named while roundtrip ok\n",
    ),
    (
        "fixtures/code-roundtrip/named-literal-target-roundtrip.jai",
        "named literal targets ok\n",
    ),
    ("fixtures/code-roundtrip/code-record-defaults.jai", ""),
    ("fixtures/code-roundtrip/inherited-record-defaults.jai", ""),
    (
        "fixtures/code-roundtrip/compiler-derived-record-defaults.jai",
        "",
    ),
    (
        "fixtures/code-roundtrip/late-import-conditional-procedure.jai",
        "",
    ),
];

fn source(path: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests")
        .join(path)
}

fn success(output: Output, expected: &str) {
    assert!(
        output.status.success(),
        "{:?}: {}",
        output.status,
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&output.stdout), expected);
}

#[test]
fn filtered_parameters_and_caller_visible_while_roundtrip_in_the_interpreter() {
    for &(path, expected) in CASES {
        success(
            Command::new(env!("CARGO_BIN_EXE_jaic"))
                .arg(source(path))
                .arg("-interp")
                .output()
                .unwrap(),
            expected,
        );
    }
}

#[test]
fn explicitly_reset_record_kind_is_rejected_without_shape_guessing() {
    let output = Command::new(env!("CARGO_BIN_EXE_jaic"))
        .arg(source("fixtures/code-roundtrip/code-record-reset-tag.jai"))
        .arg("-check")
        .output()
        .unwrap();
    assert!(!output.status.success());
    assert!(
        String::from_utf8_lossy(&output.stderr)
            .contains("compiler_get_code cannot print this syntax node kind")
    );
}

#[cfg(feature = "llvm")]
#[test]
fn graph_roundtrips_execute_native_at_o0_and_o2() {
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("code-roundtrips");
    std::fs::create_dir_all(&dir).unwrap();
    for (i, &(path, expected)) in CASES.iter().enumerate() {
        for opt in ["-O0", "-O2"] {
            let executable = dir.join(format!("case-{i}{opt}"));
            success(
                Command::new(env!("CARGO_BIN_EXE_jaic"))
                    .arg(source(path))
                    .args([opt, "-exe"])
                    .arg(&executable)
                    .output()
                    .unwrap(),
                "",
            );
            success(Command::new(executable).output().unwrap(), expected);
        }
    }
}

#[cfg(feature = "llvm")]
#[test]
fn edited_parameter_filters_and_named_literal_targets_execute_as_real_wasi_commands() {
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("code-roundtrip-wasi");
    std::fs::create_dir_all(&dir).unwrap();
    for (i, (path, expected)) in [CASES[1], CASES[3], CASES[4], CASES[5], CASES[6]]
        .into_iter()
        .enumerate()
    {
        let module = dir.join(format!("syntax-{i}.wasm"));
        success(
            Command::new(env!("CARGO_BIN_EXE_jaic"))
                .arg(source(path))
                .args(["-os", "wasm", "-exe"])
                .arg(&module)
                .output()
                .unwrap(),
            "",
        );
        let runner =
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../tools/run_foundations_wasm.mjs");
        success(
            Command::new("node")
                .arg(runner)
                .arg(module)
                .output()
                .unwrap(),
            expected,
        );
    }
}
