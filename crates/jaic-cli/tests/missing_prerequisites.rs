//! A prerequisite `jaic` cannot find ends the build before any work, with an `error:` that names
//! it and a `note:` with the next step.
use std::process::Command;
use test_support::TempDir;

const JAIC: &str = env!("CARGO_BIN_EXE_jaic");

#[test]
fn a_missing_standard_library_names_the_variable_that_fixes_it() {
    let dir = TempDir::new("missing-stdlib");
    std::fs::write(dir.join("hello.jai"), "main :: () {}\n").unwrap();
    let output = Command::new(JAIC)
        .args(["-check", "hello.jai"])
        .current_dir(&dir)
        .env("JAIC_STDLIB", dir.join("nowhere"))
        .output()
        .unwrap();
    assert!(!output.status.success());
    let text = String::from_utf8_lossy(&output.stderr);
    assert!(
        text.contains("error[cli-failure]: cannot find the standard library"),
        "{text}"
    );
    assert!(text.contains("\nnote: keep the `stdlib` folder"), "{text}");
    assert!(text.contains("JAIC_STDLIB"), "{text}");
}
