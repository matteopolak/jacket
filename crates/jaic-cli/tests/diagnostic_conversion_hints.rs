//! The help on conversion and cast errors, as the command line prints it: the plain layout has
//! the advice, the rich layouts show the changed code under it. See docs/compiler/diagnostics.md.
use std::path::Path;
use std::process::Command;
use test_support::TempDir;

const JAIC: &str = env!("CARGO_BIN_EXE_jaic");

fn scratch(name: &str) -> TempDir {
    TempDir::new(name)
}

/// `jaic -check` on `source` in `dir` (as `t.jai`), with the diagnostics layout `layout` (empty
/// for the plain one).
fn check_in(dir: &Path, source: &str, layout: &str) -> String {
    std::fs::write(dir.join("t.jai"), source).unwrap();
    let mut command = Command::new(JAIC);
    command
        .args(["-check", "t.jai", "-no_color"])
        .current_dir(dir)
        .env_remove("FORCE_COLOR")
        .env_remove("CLICOLOR_FORCE")
        .env_remove("NO_COLOR")
        .env_remove("JAIC_DIAGNOSTICS");
    if !layout.is_empty() {
        command.env("JAIC_DIAGNOSTICS", layout);
    }
    String::from_utf8_lossy(&command.output().unwrap().stderr).into_owned()
}

fn check(name: &str, source: &str, layout: &str) -> String {
    check_in(&scratch(name), source, layout)
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

const MISMATCH: &str = "main :: () {\n    a: s64 = 3;\n    b: s32 = a;\n}\n";

#[test]
fn a_conversion_error_names_the_cast() {
    let plain = check("mismatch-plain", MISMATCH, "");
    assert_in_order(
        &plain,
        &[
            "t.jai:3:14: error[type-mismatch]: type mismatch: expected `s32`, found `s64`",
            "help: convert with `cast(s32)` if every value fits",
        ],
    );
}

#[test]
fn the_cast_shows_the_changed_code() {
    let rich = check("mismatch-rich", MISMATCH, "unicode");
    assert_in_order(
        &rich,
        &[
            "help: convert with `cast(s32)` if every value fits",
            " 3 ~     b: s32 = cast(s32) a;",
        ],
    );
}

#[test]
fn a_loop_over_a_number_shows_the_range() {
    let source = "main :: () {\n    n: s64 = 3;\n    for n { }\n}\n";
    let rich = check("loop-rich", source, "unicode");
    assert_in_order(
        &rich,
        &[
            "error[cannot-iterate-over]: cannot iterate over a value of type s64",
            "help: to repeat n times, loop over a range: `for 0..n - 1`",
            " 3 ~     for 0..n - 1 { }",
        ],
    );
}

#[test]
fn a_return_with_too_many_values_names_the_results() {
    let source = "f :: () -> s64 {\n    return 1, 2;\n}\nmain :: () { f(); }\n";
    let plain = check("return-plain", source, "");
    assert_in_order(
        &plain,
        &[
            "error[too-many-return-values]: too many return values (procedure returns 1)",
            "help: return only the first 1 value",
            "help: the procedure returns `s64`",
        ],
    );
}
