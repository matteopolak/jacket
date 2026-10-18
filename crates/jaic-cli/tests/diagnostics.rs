//! What `jaic` tells the user when something goes wrong: runtime check failures with their
//! call stack, compile errors with suggestions, and command-line mistakes.
//! The message style is described in docs/compiler/diagnostics.md.
#[path = "diagnostics/commands.rs"]
mod commands;
#[path = "diagnostics/resolution.rs"]
mod resolution;
#[path = "diagnostics/runtime_failures.rs"]
mod runtime_failures;

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

const JAIC: &str = env!("CARGO_BIN_EXE_jaic");

/// A scratch directory of its own for each test.
fn scratch(name: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join("diagnostics")
        .join(name);
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

/// Write `source` as `dir/name` and run `jaic` on it with the mode option `mode` (`-check`,
/// `-interp`, or empty for a build) and `args`.
fn jaic_on(dir: &Path, name: &str, source: &str, mode: &str, args: &[&str]) -> Output {
    let path = dir.join(name);
    std::fs::write(&path, source).unwrap();
    let mut command = vec![path.to_str().unwrap()];
    command.extend(Some(mode).filter(|m| !m.is_empty()));
    jaic(dir, &command, args)
}

fn jaic(dir: &Path, command: &[&str], args: &[&str]) -> Output {
    Command::new(JAIC)
        .args(command)
        .args(args)
        .current_dir(dir)
        // The plain layout, whatever the terminal running the tests.
        .env_remove("FORCE_COLOR")
        .env_remove("CLICOLOR_FORCE")
        .env_remove("JAIC_DIAGNOSTICS")
        .output()
        .unwrap()
}

fn stderr(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).into_owned()
}

/// Each needle appears in `text`, in order.
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

/// A literal format string should use as many arguments as the call passes (a warning, as in
/// Jai: compilation goes on); `\%` is a plain
/// percent sign, `%N` selects an argument, and a format that is not a literal is not checked.
#[test]
fn format_string_argument_count_is_checked() {
    let dir = scratch("format-count");
    let check = |name: &str, body: &str| {
        let source = format!("#import \"Basic\";\nmain :: () {{\n    n := 1;\n{body}\n}}\n");
        jaic_on(&dir, name, &source, "-check", &[])
    };

    let few = check("few.jai", "    print(\"% is %\\n\", n);");
    assert_eq!(few.status.code(), Some(0), "{}", stderr(&few));
    assert_in_order(
        &stderr(&few),
        &[
            "few.jai:4:5: warning: incorrect number of arguments supplied to `print`: the format string requires 2 arguments, but 1 argument is given",
            "help: pass a value for every `%`",
        ],
    );

    let many = check("many.jai", "    s := tprint(\"done\\n\", n, 2);");
    assert_eq!(many.status.code(), Some(0), "{}", stderr(&many));
    assert_in_order(
        &stderr(&many),
        &[
            "many.jai:4:10: warning: incorrect number of arguments supplied to `tprint`: the format string requires 0 arguments, but 2 arguments are given",
            "help: add a `%` for each argument",
        ],
    );

    let fine = check(
        "fine.jai",
        "    print(\"100\\% of %1 and %1: %\\n\", n, n);\n    format := \"%\";\n    print(format);\n    print(\"%00\\n\");\n    print(\"%%\\n\", n, n);",
    );
    assert_eq!(fine.status.code(), Some(0), "{}", stderr(&fine));
}

#[test]
fn statements_after_a_return_get_a_warning() {
    let dir = scratch("after-return-warning");
    let source = "\
f :: (x: int) -> int {
    if x > 0 { return 1; x += 1; }
    for 1..3 { continue; x += 1; }
    return 2;
    Constant :: 5;
}
main :: () {
    f(1);
}
";
    let output = jaic_on(&dir, "t.jai", source, "-check", &[]);
    assert!(output.status.success(), "{}", stderr(&output));
    let text = stderr(&output);
    assert_in_order(
        &text,
        &[
            "t.jai:2:26: warning: statements in block after `return` are never reached",
            "t.jai:3:26: warning: statements in block after `continue` are never reached",
        ],
    );
    assert_eq!(text.matches("never reached").count(), 2, "{text}");
}

#[test]
fn a_body_that_can_fall_off_its_end_gets_a_warning() {
    let dir = scratch("missing-return-warning");
    let source = "\
E :: enum { A; B; }
open :: (x: int) -> int {
    if x > 0 return 1;
}
all_enum :: (e: E) -> int {
    if #complete e == { case .A; return 1; case .B; return 2; }
}
with_default :: (x: int) -> int {
    if x == { case 1; return 1; case; return 2; }
}
forever :: () -> int {
    while true { if open(1) == 2 return 1; }
}
named :: () -> (r: int) { }
chosen :: () -> int {
    #if true { return 1; } else { }
}
main :: () {
    open(1); all_enum(.A); with_default(1); forever(); named(); chosen();
}
";
    let output = jaic_on(&dir, "t.jai", source, "-check", &[]);
    assert!(output.status.success(), "{}", stderr(&output));
    let text = stderr(&output);
    // `open` and `forever` (a loop whose body does not always return) warn; the switches, the
    // chosen `#if` branch and `named` (it returns its results at the end) do not.
    for at in ["t.jai:2:", "t.jai:11:"] {
        assert_in_order(
            &text,
            &[at, "warning: not all control paths return a value"],
        );
    }
    assert_eq!(text.matches("not all control paths").count(), 2, "{text}");
}

#[test]
fn a_switch_on_a_constant_checks_every_case() {
    let dir = scratch("const-switch");
    let source = "#import \"Basic\";\nMode :: enum { A; B; }\nM :: Mode.A;\nmain :: () {\n    if #complete M == {\n        case .A; print(\"a\\n\");\n        case .B; x: int = \"not an int\"; print(\"%\\n\", x);\n    }\n}\n";
    let output = jaic_on(&dir, "const_switch.jai", source, "-check", &[]);
    assert!(!output.status.success(), "{}", stderr(&output));
    assert_in_order(&stderr(&output), &["const_switch.jai:7:27", "int"]);
    // The cases that do not match count for `#complete` too.
    let source = "Mode :: enum { A; B; }\nM :: Mode.A;\nmain :: () {\n    if #complete M == {\n        case .A;\n    }\n}\n";
    let output = jaic_on(&dir, "const_complete.jai", source, "-check", &[]);
    assert!(!output.status.success(), "{}", stderr(&output));
    assert_in_order(&stderr(&output), &["has no case for .B"]);
}

#[test]
fn a_failed_initializer_reports_its_own_error_and_nothing_after_it() {
    let dir = scratch("failed_initializer");
    let source = "#import \"Basic\";\nhelper :: () {\n    owner := Nope.make();\n    size := owner.count + 1;\n    print(\"%\\n\", size);\n}\nmain :: () { helper(); }\n";
    let output = jaic_on(&dir, "main.jai", source, "-check", &[]);
    assert!(!output.status.success());
    let text = stderr(&output);
    assert_eq!(text.matches("error:").count(), 1, "{text}");
    assert!(text.contains("unknown identifier `Nope`"), "{text}");
    assert!(
        !text.contains("`owner`") && !text.contains("`size`"),
        "{text}"
    );
}
