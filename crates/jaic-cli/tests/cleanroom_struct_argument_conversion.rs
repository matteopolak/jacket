//! Execute concrete typed generic callbacks and their reflected storage on real backends.
use std::{path::PathBuf, process::Command};

fn source() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/producer-requests/typed-generic-procedure-parameter.jai")
}

fn success(command: &mut Command) {
    let output = command.output().unwrap();
    assert!(
        output.status.success(),
        "{:?}: {}",
        output.status,
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        output.stdout.is_empty(),
        "{}",
        String::from_utf8_lossy(&output.stdout)
    );
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
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("typed-generic-callbacks-native");
    std::fs::create_dir_all(&dir).unwrap();
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
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("typed-generic-callbacks-wasi");
    std::fs::create_dir_all(&dir).unwrap();
    let module = dir.join("typed-generic-callbacks.wasm");
    success(
        Command::new(env!("CARGO_BIN_EXE_jaic"))
            .arg(source())
            .args(["-os", "wasm", "-exe"])
            .arg(&module),
    );
    let runner =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../tools/run_foundations_wasm.mjs");
    success(Command::new("node").arg(runner).arg(module));
}
