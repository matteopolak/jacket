//! A wild pointer in compile-time code fails the compile with an error instead of killing jaic.
#![cfg(any(
    all(target_os = "macos", target_arch = "aarch64"),
    all(
        target_os = "linux",
        any(target_arch = "x86_64", target_arch = "aarch64")
    )
))]
use std::process::Command;
use test_support::TempDir;

fn check(name: &str, source: &str) -> (std::process::ExitStatus, String) {
    let dir = TempDir::new("wild-pointers");
    let file = dir.join(format!("{name}.jai"));
    std::fs::write(&file, source).unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_jaic"))
        .arg("-check")
        .arg(&file)
        .output()
        .unwrap();
    (
        output.status,
        String::from_utf8_lossy(&output.stderr).into_owned(),
    )
}

#[test]
fn a_wild_read_in_a_run_is_an_error() {
    let (status, err) = check(
        "read",
        "peek :: (p: *int) -> int {\n    return p.*;\n}\n#run {\n    p := cast(*int) 0xdead0000;\n    x := peek(p);\n};\nmain :: () {}\n",
    );
    assert_eq!(status.code(), Some(1), "{err}");
    assert!(
        err.contains("invalid memory access: read from address 0xdead0000 faulted"),
        "{err}"
    );
    assert!(err.contains("read.jai"), "{err}");
    assert!(
        err.contains("while running compile-time code started here"),
        "{err}"
    );
}

#[test]
fn a_wild_write_in_a_run_is_an_error() {
    let (status, err) = check(
        "write",
        "#run {\n    p := cast(*int) 0xdead0000;\n    p.* = 5;\n};\nmain :: () {}\n",
    );
    assert_eq!(status.code(), Some(1), "{err}");
    assert!(
        err.contains("invalid memory access: write to address 0xdead0000 faulted"),
        "{err}"
    );
}
