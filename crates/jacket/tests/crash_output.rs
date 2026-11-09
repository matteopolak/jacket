//! A test program killed by a signal: `jacket test` shows the tail of its output and the crash
//! handler's report, so a crash can be read from a CI log alone.
#![cfg(unix)]
use std::path::Path;
use std::process::Command;
use test_support::{TempDir, toolchain};

const PROGRAM: &str = r#"#import "Basic";
libc :: #system_library "libc";
raise :: (signal: s32) -> s32 #foreign libc;
main :: () {
    for 1..100 print("line %\n", it);
    raise(11);
}
"#;

#[test]
fn golden_killed_by_a_signal_shows_output_and_native_stack() {
    if toolchain::jaic_beside(Path::new(env!("CARGO_BIN_EXE_jacket"))).is_none() {
        return;
    }
    let project = TempDir::new("jacket-crash-golden");
    let tests = project.join("tests");
    std::fs::create_dir_all(&tests).unwrap();
    std::fs::write(project.join("jacket.toml"), "[project]\n").unwrap();
    std::fs::write(tests.join("boom.jai"), PROGRAM).unwrap();
    // An expected-output file makes it a golden test: built and run alone.
    std::fs::write(tests.join("boom.stdout"), "").unwrap();
    let out = Command::new(env!("CARGO_BIN_EXE_jacket"))
        .arg("test")
        .current_dir(&project)
        .env(
            "JAIC_STDLIB",
            Path::new(env!("CARGO_MANIFEST_DIR")).join("../../stdlib"),
        )
        .output()
        .expect("start jacket");
    let text = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(!out.status.success(), "{text}");
    assert!(text.contains("killed by signal 11"), "{text}");
    assert!(text.contains("[36 earlier lines omitted]"), "{text}");
    assert!(text.contains("line 100"), "{text}");
    assert!(text.contains("crash signal 11 at 0x"), "{text}");
    assert!(text.contains("native stack:"), "{text}");
}
