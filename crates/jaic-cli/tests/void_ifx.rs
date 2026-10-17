//! Execute the same independent discarded-void conditional on real backends.
mod common;
use common::success_silent as success;
use std::{path::PathBuf, process::Command};

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
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("void-ifx-native");
    std::fs::create_dir_all(&dir).unwrap();
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
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("void-ifx-wasi");
    std::fs::create_dir_all(&dir).unwrap();
    let module = dir.join("ifx.wasm");
    success(
        Command::new(env!("CARGO_BIN_EXE_jaic"))
            .arg(fixture())
            .args(["-os", "wasm", "-exe"])
            .arg(&module),
    );
    let runner =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../tools/run_foundations_wasm.mjs");
    success(Command::new("node").arg(runner).arg(module));
}
