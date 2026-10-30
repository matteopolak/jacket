//! How `jaic` prints the suggestions of a diagnostic (`jaic::source::Suggestion`): each is a
//! `help:` line, with the changed code under it in the rich layouts. Diagnostics without a
//! suggestion keep their plain `help:` lines. See docs/compiler/diagnostics.md.
use std::process::Command;
use test_support::TempDir;

const JAIC: &str = env!("CARGO_BIN_EXE_jaic");

fn scratch(name: &str) -> TempDir {
    TempDir::new(name)
}

/// `jaic -check` on `source`, with the diagnostics layout `layout` (empty for the plain one).
fn check(name: &str, source: &str, layout: &str) -> String {
    let dir = scratch(name);
    std::fs::write(dir.join("t.jai"), source).unwrap();
    let mut command = Command::new(JAIC);
    command
        .args(["-check", "t.jai", "-no_color"])
        .current_dir(&dir)
        .env_remove("FORCE_COLOR")
        .env_remove("CLICOLOR_FORCE")
        .env_remove("NO_COLOR")
        .env_remove("JAIC_DIAGNOSTICS");
    if !layout.is_empty() {
        command.env("JAIC_DIAGNOSTICS", layout);
    }
    String::from_utf8_lossy(&command.output().unwrap().stderr).into_owned()
}

#[track_caller]
fn assert_in_order(text: &str, needles: &[&str]) {
    let mut rest = text;
    for needle in needles {
        match rest.find(needle) {
            Some(at) => rest = &rest[at + needle.len()..],
            None => panic!("expected {needle:?} (in this order) in:\n{text}"),
        }
    }
}

const TYPO: &str =
    "#import \"Basic\";\nmain :: () {\n    counter := 1;\n    print(\"%\\n\", countr);\n}\n";

#[test]
fn a_replacement_is_a_help_line() {
    let plain = check("replacement-plain", TYPO, "");
    assert_in_order(
        &plain,
        &[
            "t.jai:4:18: error: unknown identifier `countr`",
            "help: a similar name exists: `counter`",
        ],
    );
}

#[test]
fn a_replacement_shows_the_changed_code_in_the_rich_layouts() {
    let rich = check("replacement-rich", TYPO, "unicode");
    assert_in_order(
        &rich,
        &[
            "help: a similar name exists: `counter`",
            " 4 ~     print(\"%\\n\", counter);",
        ],
    );
}

#[test]
fn an_import_is_a_help_line_and_the_other_modules_one_more() {
    let output = check("import", "main :: () {\n    x := log(2.0);\n}\n", "");
    // The first module is the suggestion, the rest are named together after it.
    assert_in_order(
        &output,
        &[
            "error: unknown identifier `log`",
            "help: `log` is declared in the `Basic` module: add `#import \"Basic\";` to this file",
            "help: other modules that declare `log`: `Math`, `Extensions/Testing`",
        ],
    );
    assert_eq!(output.matches("help:").count(), 2, "{output}");
}

#[test]
fn a_diagnostic_without_suggestions_keeps_its_plain_help() {
    let output = check(
        "plain-help",
        "main :: () {\n    x: s32 = 1;\n    y: s64 = x;\n    z: string = y;\n}\n",
        "",
    );
    assert!(output.contains("error: type mismatch"), "{output}");
    assert!(!output.contains(" ~ "), "{output}");
}
