//! Compile-time services (`Extensions/Service`) in a build: the worker is started once, answers
//! every status, survives between calls and is reported well when it crashes. The behavior is
//! described in docs/extensions/service.md.
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use test_support::TempDir;

const JAIC: &str = env!("CARGO_BIN_EXE_jaic");

fn tests_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../stdlib/Extensions/Service/tests")
}

fn scratch(name: &str) -> TempDir {
    TempDir::new(name)
}

fn jaic(dir: &Path, args: &[&str]) -> Output {
    Command::new(JAIC)
        .args(args)
        .current_dir(dir)
        .env_remove("FORCE_COLOR")
        .env_remove("CLICOLOR_FORCE")
        .env_remove("JAIC_DIAGNOSTICS")
        .output()
        .unwrap()
}

/// A program whose compile-time code asks the toy service `services/upper.jai` with `body`.
fn program(body: &str) -> String {
    let upper = tests_dir().join("services/upper.jai");
    let upper = jacket_config::slashed(&upper);
    format!(
        "#import \"Basic\";\nService :: #import \"Extensions/Service\";\n\
         UPPER :: \"{upper}\";\n{body}\nmain :: () {{}}\n"
    )
}

#[test]
fn every_status_reaches_compile_time_code() {
    let dir = tests_dir();
    let output = jaic(&dir, &["call.jai", "-interp"]);
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        stdout,
        "OK HELLO\nOK AGAIN\nFAILED no table\nUNAVAILABLE no database\nFAILED\nUNAVAILABLE\n"
    );
}

#[test]
fn a_crash_is_reported_with_the_service_output_and_not_retried_at_once() {
    let dir = scratch("crash");
    std::fs::write(
        dir.join("case.jai"),
        program(
            "#run {\n\
             s1, r1 := Service.call(UPPER, \"crash\");\n\
             s2, r2 := Service.call(UPPER, \"after\");\n\
             print(\"%|%\\n%|%\\n\", s1, r1, s2, r2);\n}",
        ),
    )
    .unwrap();
    let output = jaic(&dir, &["case.jai", "-check"]);
    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut lines = stdout.lines();
    let first = lines.next().unwrap_or_default();
    assert!(first.starts_with("FAILED|the service stopped"), "{stdout}");
    let second = lines.next().unwrap_or_default();
    assert!(
        second.starts_with("FAILED|the service failed and is not restarted for 1 s"),
        "{stdout}"
    );
}

#[test]
fn an_unknown_result_is_an_error_in_a_build() {
    let dir = scratch("unknown");
    std::fs::write(
        dir.join("case.jai"),
        program("#run Service.unknown(\"no database configured\");"),
    )
    .unwrap();
    let output = jaic(&dir, &["case.jai", "-check"]);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_eq!(output.status.code(), Some(1), "{stderr}");
    let first = stderr.lines().next().unwrap_or_default();
    assert!(
        first.starts_with("case.jai:4:") && first.ends_with("error: no database configured"),
        "{stderr}"
    );
}
