//! Insertion lexical-shape controls.
mod common;
use common::success_with_stdout as success;
use std::{path::PathBuf, process::Command};

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn source(name: &str) -> PathBuf {
    if name == "human" {
        root().join("tests/fixtures/insert-scope/main.jai")
    } else {
        root().join(format!(
            "tests/cleanroom-language/insert-declaration-scope/{name}.jai"
        ))
    }
}

const POSITIVE: &[(&str, &str)] = &[
    ("human", "b: 2\n"),
    ("main", "ok\n"),
    ("roundtrip", "roundtrip ok\n"),
];

const NEGATIVE: &[(&str, &str)] = &[
    ("braced-private", "unknown identifier"),
    ("double-braced-private", "unknown identifier"),
    ("constant-before-insert", "unknown identifier"),
    ("nested-private", "unknown identifier"),
    ("duplicate", "already declared"),
];

#[test]
fn exact_human_and_scope_roundtrips_check_and_interpret() {
    for &(name, expected) in POSITIVE {
        success(
            Command::new(env!("CARGO_BIN_EXE_jaic"))
                .arg(source(name))
                .arg("-check")
                .output()
                .unwrap(),
            "",
        );
        success(
            Command::new(env!("CARGO_BIN_EXE_jaic"))
                .arg(source(name))
                .arg("-interp")
                .output()
                .unwrap(),
            expected,
        );
    }
}

#[test]
fn explicit_block_privacy_preinsert_constants_and_duplicates_reject() {
    for &(name, diagnostic) in NEGATIVE {
        for mode in ["-check", "-interp"] {
            let output = Command::new(env!("CARGO_BIN_EXE_jaic"))
                .arg(source(name))
                .arg(mode)
                .output()
                .unwrap();
            assert!(!output.status.success(), "{name} {mode} must reject");
            assert!(
                String::from_utf8_lossy(&output.stderr).contains(diagnostic),
                "{name}: {}",
                String::from_utf8_lossy(&output.stderr)
            );
        }
    }
}

#[cfg(feature = "llvm")]
#[test]
fn insertion_scope_runs_native_o0_o2_and_wasi() {
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("insert-scope");
    std::fs::create_dir_all(&dir).unwrap();
    for &(name, diagnostic) in NEGATIVE {
        for opt in ["-O0", "-O2"] {
            let output = Command::new(env!("CARGO_BIN_EXE_jaic"))
                .arg(source(name))
                .args([opt, "-exe"])
                .arg(dir.join(format!("reject-{name}{opt}")))
                .output()
                .unwrap();
            assert!(!output.status.success(), "{name} {opt} must reject");
            assert!(
                String::from_utf8_lossy(&output.stderr).contains(diagnostic),
                "{name}: {}",
                String::from_utf8_lossy(&output.stderr)
            );
        }
    }
    for &(name, expected) in POSITIVE {
        for opt in ["-O0", "-O2"] {
            let executable = dir.join(format!("{name}{opt}"));
            success(
                Command::new(env!("CARGO_BIN_EXE_jaic"))
                    .arg(source(name))
                    .args([opt, "-exe"])
                    .arg(&executable)
                    .output()
                    .unwrap(),
                "",
            );
            success(Command::new(executable).output().unwrap(), expected);
            let module = dir.join(format!("{name}{opt}.wasm"));
            success(
                Command::new(env!("CARGO_BIN_EXE_jaic"))
                    .arg(source(name))
                    .args(["-os", "wasm", opt, "-exe"])
                    .arg(&module)
                    .output()
                    .unwrap(),
                "",
            );
            success(
                Command::new("node")
                    .arg(root().join("tools/run_foundations_wasm.mjs"))
                    .arg(module)
                    .output()
                    .unwrap(),
                expected,
            );
        }
    }
}
