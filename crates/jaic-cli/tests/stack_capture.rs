//! Current-stack capture includes its own helper frame; disabled tracing stays empty.
use std::{path::Path, process::Command};
#[cfg(feature = "llvm")]
use test_support::TempDir;

#[test]
fn current_stack_capture_survives_return_and_keeps_the_helper_frame() {
    let source = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/compiler/regressions/stack-capture-frame.jai");
    let result = Command::new(env!("CARGO_BIN_EXE_jaic"))
        .arg(source)
        .arg("-interp")
        .output()
        .unwrap();
    assert!(result.status.success(), "{result:?}");
}

#[test]
#[cfg(feature = "llvm")]
fn native_capture_keeps_frames_and_disabled_tracing_stays_empty() {
    let dir = TempDir::new("native-stack-capture");
    let source = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/compiler/regressions/stack-capture-frame.jai");
    for optimization in ["-O0", "-O2"] {
        let executable = dir.join(if cfg!(windows) {
            "frames.exe"
        } else {
            "frames"
        });
        let build = Command::new(env!("CARGO_BIN_EXE_jaic"))
            .arg(&source)
            .arg(optimization)
            .arg("-exe")
            .arg(&executable)
            .output()
            .unwrap();
        assert!(build.status.success(), "{build:?}");
        let run = Command::new(executable).output().unwrap();
        assert!(run.status.success(), "{run:?}");
    }
    let source = dir.join("disabled.jai");
    std::fs::write(&source,"#import \"Basic\"; main :: () { trace := pack_stack_trace(); defer free(trace.data); if trace.count != 0 exit(1); }").unwrap();
    let executable = dir.join(if cfg!(windows) {
        "disabled.exe"
    } else {
        "disabled"
    });
    let build = Command::new(env!("CARGO_BIN_EXE_jaic"))
        .arg(source)
        .arg("-o")
        .arg("-exe")
        .arg(&executable)
        .output()
        .unwrap();
    assert!(build.status.success(), "{build:?}");
    let run = Command::new(executable).output().unwrap();
    assert!(run.status.success(), "{run:?}");
}
