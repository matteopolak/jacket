//! A wild pointer in compile-time code is an error in the editor, and the server keeps working.
#![cfg(any(
    all(target_os = "macos", target_arch = "aarch64"),
    all(
        target_os = "linux",
        any(target_arch = "x86_64", target_arch = "aarch64")
    )
))]
use jai_language_server::DiagnosticSeverity;
mod common;

#[test]
fn a_wild_read_is_reported_at_the_run_and_the_session_survives() {
    let mut s = common::session();
    s.open(
        common::uri(),
        1,
        "#run {\n    p := cast(*int) 0xdead0000;\n    x := p.*;\n};\nmain :: () {}\n".into(),
    )
    .unwrap();
    let errors: Vec<_> = s
        .diagnostics(&common::uri())
        .unwrap()
        .into_iter()
        .filter(|d| d.severity == DiagnosticSeverity::Error)
        .collect();
    assert_eq!(errors.len(), 1, "{errors:#?}");
    assert!(
        errors[0]
            .message
            .contains("invalid memory access: read from address 0xdead0000 faulted"),
        "{}",
        errors[0].message
    );
    // The process lives on: a fresh session checks fixed code.
    let mut fresh = common::session();
    fresh
        .open(common::uri(), 1, "#run 1 + 2;\nmain :: () {}\n".into())
        .unwrap();
    let errors = fresh
        .diagnostics(&common::uri())
        .unwrap()
        .into_iter()
        .filter(|d| d.severity == DiagnosticSeverity::Error)
        .count();
    assert_eq!(errors, 0);
}
