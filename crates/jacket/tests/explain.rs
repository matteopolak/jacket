//! `jacket explain`: what a diagnostic code means, the list of all of them, and the nudge for a
//! mistyped one.
use std::process::{Command, Output};

fn explain(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_jacket"))
        .arg("explain")
        .args(args)
        .output()
        .expect("run jacket")
}

fn stdout(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout).into_owned()
}

fn stderr(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).into_owned()
}

#[test]
fn a_code_prints_its_summary_explanation_and_example() {
    let output = explain(&["cast-shift-precedence"]);
    assert!(output.status.success(), "{}", stderr(&output));
    let text = stdout(&output);
    assert!(
        text.starts_with("cast-shift-precedence (warning, checks)\n"),
        "{text}"
    );
    assert!(
        text.contains("a cast binds more loosely than a shift"),
        "{text}"
    );
    assert!(
        text.contains("\nExample:\n\n    low: u8 = 0x12;\n"),
        "{text}"
    );
}

#[test]
fn the_header_of_a_diagnostic_can_be_pasted_whole() {
    let output = explain(&["error[scope-file-name]"]);
    assert!(output.status.success(), "{}", stderr(&output));
    assert!(stdout(&output).starts_with("scope-file-name (error, checker/scope)\n"));
}

#[test]
fn a_code_without_an_explanation_still_has_its_summary() {
    let output = explain(&["unknown-library"]);
    assert!(output.status.success(), "{}", stderr(&output));
    let text = stdout(&output);
    assert!(
        text.starts_with("unknown-library (error, names)\n"),
        "{text}"
    );
    assert!(!text.contains("Example:"), "{text}");
}

#[test]
fn a_leniency_says_the_official_compiler_rejects_it() {
    let output = explain(&["declaration-hides-import"]);
    assert!(output.status.success(), "{}", stderr(&output));
    assert!(stdout(&output).contains("the official Jai compiler rejects this"));
}

#[test]
fn no_code_lists_them_grouped_by_area() {
    let output = explain(&[]);
    assert!(output.status.success(), "{}", stderr(&output));
    let text = stdout(&output);
    assert!(text.contains("\nnames\n  "), "{text}");
    assert!(text.contains("\n  type-mismatch "), "{text}");
    assert!(
        text.ends_with("`jacket explain <code>` says more about one.\n"),
        "{text}"
    );
}

#[test]
fn a_mistyped_code_suggests_the_nearest() {
    let output = explain(&["type-mismtach"]);
    assert_eq!(output.status.code(), Some(2));
    assert_eq!(
        stderr(&output),
        "jacket: no diagnostic code `type-mismtach`\nhelp: did you mean `type-mismatch`?\n"
    );
    let far = explain(&["zzzzzzzzzz"]);
    assert_eq!(far.status.code(), Some(2));
    assert!(
        stderr(&far).contains("help: `jacket explain` lists them all"),
        "{}",
        stderr(&far)
    );
}
