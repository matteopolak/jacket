//! What `jaic` tells the user when something goes wrong: runtime check failures with their
//! call stack, compile errors with suggestions, and command-line mistakes.
//! The message style is described in docs/compiler/diagnostics.md.
use test_support::TempDir;
#[path = "diagnostics/commands.rs"]
mod commands;
#[path = "diagnostics/resolution.rs"]
mod resolution;
#[path = "diagnostics/runtime_failures.rs"]
mod runtime_failures;

use std::path::Path;
use std::process::{Command, Output};

const JAIC: &str = env!("CARGO_BIN_EXE_jaic");

/// A scratch directory of its own for each test.
fn scratch(name: &str) -> TempDir {
    TempDir::new(name)
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
fn a_float_literal_that_float32_changes_gets_a_warning() {
    let dir = scratch("float32-rounding-warning");
    let source = "\
main :: () {
    a := 12342345234.0;
    b: float = 16777217.0;
    c: float = 0.000000001;
    d := 0.1;
    e: float = 1.0 / 3.0;
    f: float64 = 16777217.0;
}
";
    let output = jaic_on(&dir, "t.jai", source, "-check", &[]);
    assert!(output.status.success(), "{}", stderr(&output));
    let text = stderr(&output);
    assert_in_order(
        &text,
        &[
            "t.jai:2:10: warning: this literal is 12342345728 as a float32",
            "t.jai:3:16: warning: this literal is 16777216 as a float32",
        ],
    );
    assert_eq!(text.matches("as a float32").count(), 2, "{text}");
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

/// A metaprogram's report at a byte inside a string literal (a word of SQL, say) underlines
/// that word, not the rest of the line.
#[test]
fn report_inside_a_string_underlines_the_word() {
    let dir = scratch("report-in-string");
    // The line the report points at, and the carets under it.
    let report = |name: &str, text: &str, line: u32, column: u32| {
        let source = dir.join(name);
        let program = format!(
            "#import \"Compiler\";\n{text}\n#run compiler_report(\"no such column\", .{{ fully_pathed_filename = #file, line_number = {line}, character_number = {column} }});\n"
        );
        std::fs::write(&source, program).unwrap();
        let output = Command::new(JAIC)
            .arg("-check")
            .arg(&source)
            .output()
            .unwrap();
        let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
        assert!(!output.status.success(), "{stderr}");
        let lines: Vec<&str> = stderr.lines().collect();
        let at = lines
            .iter()
            .position(|l| l.contains("error: no such column"))
            .unwrap_or_else(|| panic!("{stderr}"));
        (
            lines[at + 1].trim().to_string(),
            lines[at + 2].trim().to_string(),
            stderr,
        )
    };
    let (line, carets, stderr) = report("one.jai", "SQL :: \"select id, emal from users\";", 2, 20);
    assert_eq!(
        (line.as_str(), carets.as_str()),
        ("SQL :: \"select id, emal from users\";", "^^^^"),
        "{stderr}"
    );
    let (line, carets, stderr) = report(
        "here.jai",
        "TEXT :: #string DONE\nselect id\nfrom users where emial = 1\nDONE",
        4,
        18,
    );
    assert_eq!(
        (line.as_str(), carets.as_str()),
        ("from users where emial = 1", "^^^^^"),
        "{stderr}"
    );
}

#[test]
fn a_leniency_in_the_program_file_warns_with_the_note_and_the_exit_status_stays_zero() {
    let dir = scratch("leniency-note");
    let output = jaic_on(
        &dir,
        "main.jai",
        "S :: struct { a: u16; #place a; b: float64; }\nmain :: () { s: S; s.b = 1; }\n",
        "-check",
        &[],
    );
    assert!(output.status.success(), "{}", stderr(&output));
    let text = stderr(&output);
    assert_in_order(
        &text,
        &[
            "warning: `#place` is a deprecated directive",
            "note: the official Jai compiler rejects this; Jacket accepts it but may stop in a future release",
            "help: write `#overlay(member)`",
        ],
    );
}

#[test]
fn a_leniency_in_an_imported_module_is_not_reported() {
    let dir = scratch("leniency-module");
    std::fs::create_dir_all(dir.join("Lib")).unwrap();
    std::fs::write(
        dir.join("Lib/module.jai"),
        "S :: struct { a: u16; #place a; b: float64; }\ngo :: () { s: S; s.b = 1; }\n",
    )
    .unwrap();
    let output = jaic_on(
        &dir,
        "main.jai",
        "#import \"Lib\";\nmain :: () { go(); }\n",
        "-check",
        &["-I", dir.to_str().unwrap()],
    );
    assert!(output.status.success(), "{}", stderr(&output));
    assert!(
        !stderr(&output).contains("official Jai compiler"),
        "{}",
        stderr(&output)
    );
}

#[test]
fn a_body_constant_used_above_its_declaration_is_an_error() {
    let dir = scratch("use-before-declaration");
    let output = jaic_on(
        &dir,
        "main.jai",
        "main :: () { foo(); foo :: () { } }\n",
        "-check",
        &[],
    );
    assert!(!output.status.success());
    let text = stderr(&output);
    assert_in_order(
        &text,
        &[
            "error: `foo` is used above its declaration",
            "declared here",
            "help: move the declaration of `foo` above its first use",
        ],
    );
}

#[test]
fn a_body_constant_used_above_its_declaration_is_an_error_in_a_module_too() {
    let dir = scratch("use-before-declaration-module");
    std::fs::create_dir_all(dir.join("Lib")).unwrap();
    std::fs::write(
        dir.join("Lib/module.jai"),
        "go :: () { foo(); foo :: () { } }\n",
    )
    .unwrap();
    let output = jaic_on(
        &dir,
        "main.jai",
        "#import \"Lib\";\nmain :: () { go(); }\n",
        "-check",
        &["-I", dir.to_str().unwrap()],
    );
    assert!(!output.status.success());
    assert!(
        stderr(&output).contains("`foo` is used above its declaration"),
        "{}",
        stderr(&output)
    );
}
