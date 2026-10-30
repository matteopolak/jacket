//! Execute concrete typed generic callbacks and their reflected storage on real backends.
use std::{path::PathBuf, process::Command};
use test_support::process::success_silent as success;
use test_support::{TempDir, toolchain};

fn source() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/fixtures/struct-arguments/typed-generic-procedure-parameter.jai")
}

#[test]
fn typed_generic_callbacks_execute_in_interpreter() {
    success(
        Command::new(env!("CARGO_BIN_EXE_jaic"))
            .arg(source())
            .arg("-interp"),
    );
}

#[cfg(feature = "llvm")]
#[test]
fn typed_generic_callbacks_execute_native_at_o0_and_o2() {
    let dir = TempDir::new("typed-generic-callbacks-native");
    for opt in ["-O0", "-O2"] {
        let executable = dir.join(format!("typed-generic-callbacks{opt}"));
        success(
            Command::new(env!("CARGO_BIN_EXE_jaic"))
                .arg(source())
                .args([opt, "-exe"])
                .arg(&executable),
        );
        success(&mut Command::new(executable));
    }
}

#[cfg(feature = "llvm")]
#[test]
fn typed_generic_callbacks_execute_as_wasi() {
    if !toolchain::require_all(&["node"]) {
        return;
    }
    let dir = TempDir::new("typed-generic-callbacks-wasi");
    let module = dir.join("typed-generic-callbacks.wasm");
    success(
        Command::new(env!("CARGO_BIN_EXE_jaic"))
            .arg(source())
            .args(["-os", "wasm", "-exe"])
            .arg(&module),
    );
    let runner = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../tools/browser/run_foundations_wasm.mjs");
    success(toolchain::command("node").arg(runner).arg(module));
}
