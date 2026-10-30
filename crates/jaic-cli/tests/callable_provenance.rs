//! Callable declaration provenance on actual source-built execution backends.
use std::{path::PathBuf, process::Command};
use test_support::process::success_silent as success;
use test_support::{TempDir, toolchain};

fn source() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/fixtures/callable-provenance/callable-declaration-provenance.jai")
}

#[test]
fn callable_headers_execute_in_interpreter() {
    success(
        Command::new(env!("CARGO_BIN_EXE_jaic"))
            .arg(source())
            .arg("-interp"),
    );
}

#[cfg(feature = "llvm")]
#[test]
fn callable_headers_execute_native_at_o0_and_o2() {
    let dir = TempDir::new("callable-provenance-native");
    for opt in ["-O0", "-O2"] {
        let executable = dir.join(format!("callable-headers{opt}"));
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
fn callable_headers_execute_as_wasi() {
    if !toolchain::require_all(&["node"]) {
        return;
    }
    let dir = TempDir::new("callable-provenance-wasi");
    let module = dir.join("callable-headers.wasm");
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
