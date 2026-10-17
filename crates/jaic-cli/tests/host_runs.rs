//! Isolated host values survive source checking and actual target execution.
mod common;
use common::success_silent as success;
use std::{path::PathBuf, process::Command};

fn fixture() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/fixtures/host-runs/host-result-transfer.jai")
}

#[cfg(feature = "llvm")]
#[test]
fn workspace_host_clock_produces_a_linux_x64_object() {
    let tests = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../tests/fixtures/host-runs");
    let directory = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("host-clock-object");
    std::fs::create_dir_all(&directory).unwrap();
    success(
        Command::new(env!("CARGO_BIN_EXE_jaic"))
            .arg(tests.join("host-clock-workspace.jai"))
            .arg("-")
            .arg(&directory)
            .arg(
                tests
                    .join("isolated-host-clock-result.jai")
                    .canonicalize()
                    .unwrap(),
            ),
    );
    let object = std::fs::read(directory.join("host-clock-target")).unwrap();
    assert_eq!(&object[..4], b"\x7fELF");
    assert_eq!(object[4], 2, "64-bit object");
    assert_eq!(
        u16::from_le_bytes([object[16], object[17]]),
        1,
        "relocatable object"
    );
    assert_eq!(
        u16::from_le_bytes([object[18], object[19]]),
        62,
        "x86-64 target"
    );
}

#[test]
fn host_values_runs_in_the_interpreter() {
    success(
        Command::new(env!("CARGO_BIN_EXE_jaic"))
            .arg(fixture())
            .arg("-interp"),
    );
}

#[cfg(feature = "llvm")]
#[test]
fn host_values_runs_native_at_o0_and_o2() {
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("host-values-native");
    std::fs::create_dir_all(&dir).unwrap();
    for opt in ["-O0", "-O2"] {
        let exe = dir.join(format!("host{opt}"));
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
fn host_values_runs_as_a_real_wasi_command() {
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("host-values-wasi");
    std::fs::create_dir_all(&dir).unwrap();
    let module = dir.join("host.wasm");
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
