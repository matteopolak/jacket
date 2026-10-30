//! Execute the same independent discarded-void conditional on real backends.
use std::{path::PathBuf, process::Command};
use test_support::process::success_silent as success;
use test_support::{TempDir, toolchain};

fn fixture() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/fixtures/void-ifx/ifx-discarded-void.jai")
}

#[test]
fn void_ifx_runs_in_the_interpreter() {
    success(
        Command::new(env!("CARGO_BIN_EXE_jaic"))
            .arg(fixture())
            .arg("-interp"),
    );
}

#[cfg(feature = "llvm")]
#[test]
fn void_ifx_runs_native_at_o0_and_o2() {
    let dir = TempDir::new("void-ifx-native");
    for opt in ["-O0", "-O2"] {
        let exe = dir.join(format!("ifx{opt}"));
        success(
            Command::new(env!("CARGO_BIN_EXE_jaic"))
                .arg(fixture())
                .args([opt, "-exe"])
                .arg(&exe),
        );
        success(&mut Command::new(exe));
    }
}

#[cfg(feature = "llvm")]
#[test]
fn void_ifx_runs_as_a_real_wasi_command() {
    if !toolchain::require_all(&["node"]) {
        return;
    }
    let dir = TempDir::new("void-ifx-wasi");
    let module = dir.join("ifx.wasm");
    success(
        Command::new(env!("CARGO_BIN_EXE_jaic"))
            .arg(fixture())
            .args(["-os", "wasm", "-exe"])
            .arg(&module),
    );
    let runner = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../tools/browser/run_foundations_wasm.mjs");
    success(toolchain::command("node").arg(runner).arg(module));
}
