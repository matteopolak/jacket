//! What `jaic` tells the user when something goes wrong: runtime check failures with their
//! call stack, compile errors with suggestions, and command-line mistakes.
//! The message style is described in docs/compiler/diagnostics.md.
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

/// Write `source` as `dir/name` and run `jaic <command>` on it with `args`.
fn jaic_on(dir: &Path, name: &str, source: &str, command: &str, args: &[&str]) -> Output {
    let path = dir.join(name);
    std::fs::write(&path, source).unwrap();
    jaic(dir, &[command, path.to_str().unwrap()], args)
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

#[test]
fn runtime_bounds_check_names_the_check_and_the_call_stack() {
    let dir = scratch("bounds");
    let output = jaic_on(
        &dir,
        "bounds.jai",
        "get :: (a: [] int, i: int) -> int {\n    return a[i];\n}\nmain :: () {\n    arr := int.[1, 2, 3];\n    get(arr, 5);\n}\n",
        "run",
        &[],
    );
    assert_eq!(output.status.code(), Some(1));
    let text = stderr(&output);
    assert_in_order(
        &text,
        &[
            "bounds.jai:2:5: error: runtime error: array bounds check failed: index 5 is outside an array of 3 elements",
            "    return a[i];",
            "note: call stack (innermost first):",
            "`get` at ",
            "bounds.jai:2",
            "`main` at ",
            "bounds.jai:6",
            "help: valid indices are 0 up to",
        ],
    );
    // The runtime's own entry procedures are not part of the story.
    assert!(!text.contains("Runtime_Support.jai"), "{text}");
    assert!(!text.contains("compiler-generated"), "{text}");
}

#[test]
fn failure_inside_the_stdlib_points_at_the_users_call() {
    let dir = scratch("stdlib-frame");
    let output = jaic_on(
        &dir,
        "copy.jai",
        "#import \"Basic\";\nmain :: () {\n    s: string;\n    s.count = 4;\n    t := copy_string(s);\n}\n",
        "run",
        &[],
    );
    assert_eq!(output.status.code(), Some(1));
    assert_in_order(
        &stderr(&output),
        &[
            "copy.jai:5:5: error: runtime error: null pointer dereference: memcpy through a null pointer",
            "    t := copy_string(s);",
            "note: this call failed inside `copy_string` at ",
            "Simple_String.jai",
            "note: call stack (innermost first):",
            "help: check the pointer against null",
        ],
    );
}

#[test]
fn null_pointer_and_missing_return_and_recursion() {
    let dir = scratch("checks");
    let null = jaic_on(
        &dir,
        "null.jai",
        "P :: struct { x: int; }\nmain :: () {\n    p: *P;\n    x := p.x;\n}\n",
        "run",
        &[],
    );
    assert_in_order(
        &stderr(&null),
        &["null.jai:4:5: error: runtime error: null pointer dereference: read through a null pointer"],
    );
    let missing = jaic_on(
        &dir,
        "sign.jai",
        "sign :: (x: int) -> int {\n    if x > 0 return 1;\n}\nmain :: () {\n    sign(0);\n}\n",
        "run",
        &[],
    );
    assert_in_order(
        &stderr(&missing),
        &[
            "error: runtime error: `sign` reached the end of its body without returning a value",
            "help: every path through a procedure with results must end in `return`",
        ],
    );
    let deep = jaic_on(
        &dir,
        "deep.jai",
        "down :: (n: int) -> int {\n    return down(n + 1) + 1;\n}\nmain :: () {\n    down(0);\n}\n",
        "run",
        &[],
    );
    let text = stderr(&deep);
    assert_in_order(
        &text,
        &[
            "error: runtime error: stack overflow (recursion too deep)",
            "`down` at ",
            "... the same call ",
            " more frames",
        ],
    );
    assert!(text.lines().count() < 20, "{text}");
}

#[test]
fn compile_time_failure_shows_user_code_and_the_run_site() {
    let dir = scratch("run-site");
    let output = jaic_on(
        &dir,
        "ct.jai",
        "get :: (a: [] int, i: int) -> int {\n    return a[i];\n}\nX :: #run get(int.[1, 2, 3], 7);\nmain :: () {}\n",
        "check",
        &[],
    );
    assert_eq!(output.status.code(), Some(1));
    let text = stderr(&output);
    assert_in_order(
        &text,
        &[
            "ct.jai:2:5: error: error during compile-time execution: array bounds check failed: index 7",
            "ct.jai:4:6: note: while running compile-time code started here",
            "X :: #run get(int.[1, 2, 3], 7);",
            "`get` at ",
            "the `#run` code",
        ],
    );
    assert!(!text.contains("while executing"), "{text}");
}

#[test]
fn failed_assert_reports_the_users_line_not_debug_break() {
    let dir = scratch("assert");
    let output = jaic_on(
        &dir,
        "a.jai",
        "#import \"Basic\";\nmain :: () {\n    x := 3;\n    assert(x == 4, \"x was %\", x);\n}\n",
        "run",
        &[],
    );
    assert_eq!(output.status.code(), Some(1));
    let text = stderr(&output);
    assert_in_order(
        &text,
        &[
            "a.jai:4,5: Assertion failed: x was 3",
            "Stack trace:",
            "a.jai:4:5: error: runtime error: assertion failed (message and stack trace above)",
            "    assert(x == 4, \"x was %\", x);",
        ],
    );
    assert!(!text.contains("debug_break"), "{text}");
}

#[test]
fn unknown_identifier_suggests_a_visible_name() {
    let dir = scratch("unknown-name");
    let output = jaic_on(
        &dir,
        "u.jai",
        "#import \"Basic\";\nmain :: () {\n    counter := 1;\n    print(\"%\\n\", countr);\n}\n",
        "check",
        &[],
    );
    assert_in_order(
        &stderr(&output),
        &["u.jai:4:18: error: unknown identifier", "countr", "help: a similar name exists: `counter`"],
    );
    let output = jaic_on(
        &dir,
        "p.jai",
        "#import \"Basic\";\nmain :: () {\n    prnt(\"x\\n\");\n}\n",
        "check",
        &[],
    );
    assert_in_order(&stderr(&output), &["help: a similar name exists: `print`"]);
    let output = jaic_on(&dir, "n.jai", "main :: () { x := qqqqqq; }\n", "check", &[]);
    assert!(!stderr(&output).contains("help:"), "{}", stderr(&output));
}
