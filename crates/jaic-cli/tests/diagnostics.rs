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
        &[
            "null.jai:4:5: error: runtime error: null pointer dereference: read through a null pointer",
        ],
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
    assert_eq!(
        text.lines().next().map(|l| l.rsplit('/').next().unwrap()),
        Some("a.jai:4:5: error: runtime error: assertion failed: x was 3"),
        "{text}"
    );
    assert_in_order(&text, &["    assert(x == 4, \"x was %\", x);"]);
    for noise in [
        "debug_break",
        "Stack trace",
        "assert_helper",
        "Runtime_Support",
        ",5:",
    ] {
        assert!(!text.contains(noise), "{noise}: {text}");
    }
    // Without a message, the condition from the source; the callers below it.
    let output = jaic_on(
        &dir,
        "b.jai",
        "#import \"Basic\";\nload :: (ok: bool) {\n\tassert(ok);\n}\ninit :: () { load(false); }\nmain :: () { init(); }\n",
        "run",
        &[],
    );
    assert_in_order(
        &stderr(&output),
        &[
            "b.jai:3:2: error: runtime error: assertion failed: `ok` is false",
            "\tassert(ok);",
            "note: call stack (innermost first):",
            "`load` at ",
            "`init` at ",
            "`main` at ",
        ],
    );
    // An assert inside the stdlib: the user's call is the primary location.
    let output = jaic_on(
        &dir,
        "c.jai",
        "#import \"Basic\";\nmain :: () {\n    a: [..] int;\n    array_add(*a, 1);\n    array_unordered_remove_by_index(*a, 5);\n}\n",
        "run",
        &[],
    );
    let text = stderr(&output);
    assert_in_order(
        &text,
        &[
            "c.jai:5:5: error: runtime error: assertion failed: ",
            "note: an assertion failed inside `array_unordered_remove_by_index` at ",
        ],
    );
    assert!(!text.contains("#1"), "{text}");
    // In compile-time code too.
    let output = jaic_on(
        &dir,
        "d.jai",
        "#import \"Basic\";\n#run {\n    assert(1 == 2, \"compile-time check\");\n};\nmain :: () {}\n",
        "check",
        &[],
    );
    assert_in_order(
        &stderr(&output),
        &[
            "d.jai:3:5: error: error during compile-time execution: assertion failed: compile-time check",
            "d.jai:2:1: note: while running compile-time code started here",
        ],
    );
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
        &[
            "u.jai:4:18: error: unknown identifier",
            "countr",
            "help: a similar name exists: `counter`",
        ],
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

#[test]
fn missing_module_lists_where_it_looked_and_a_close_name() {
    let dir = scratch("missing-module");
    let output = jaic_on(
        &dir,
        "m.jai",
        "#import \"Basik\";\nmain :: () {}\n",
        "check",
        &[],
    );
    assert_eq!(output.status.code(), Some(1));
    let text = stderr(&output);
    assert_in_order(
        &text,
        &[
            "m.jai:1:1: error: module `Basik` not found",
            "note: looked for `Basik.jai` and `Basik/module.jai` in:",
            "/modules",
            "/stdlib",
            "help: a module with a similar name exists: `Basic`",
        ],
    );
    assert!(!text.contains("/../"), "{text}");
    // Nothing close: where modules go.
    let output = jaic_on(
        &dir,
        "n.jai",
        "#import \"Zzyzx_Engine\";\nmain :: () {}\n",
        "check",
        &[],
    );
    assert_in_order(
        &stderr(&output),
        &["help: a module of your own goes in a `modules` folder"],
    );
}

#[test]
fn missing_load_file_says_where_it_looked() {
    let dir = scratch("missing-load");
    std::fs::write(dir.join("helper.jai"), "helper :: () {}\n").unwrap();
    let output = jaic_on(
        &dir,
        "l.jai",
        "#load \"helpers.jai\";\nmain :: () {}\n",
        "check",
        &[],
    );
    assert_eq!(output.status.code(), Some(1));
    assert_in_order(
        &stderr(&output),
        &[
            "l.jai:1:7: error: file `helpers.jai` does not exist",
            "note: looked for ",
            "helpers.jai",
            "help: a file with a similar name exists: `helper.jai`",
            "help: `#load` paths are relative to the file that loads them",
        ],
    );
}

#[test]
fn input_file_problems() {
    let dir = scratch("input");
    std::fs::write(dir.join("game.jai"), "main :: () {}\n").unwrap();
    std::fs::create_dir_all(dir.join("proj")).unwrap();
    std::fs::write(dir.join("proj/first.jai"), "main :: () {}\n").unwrap();
    let cases: &[(&[&str], &[&str])] = &[
        (
            &["run", "gmae.jai"],
            &[
                "error: file `gmae.jai` does not exist",
                "help: did you mean `game.jai`?",
            ],
        ),
        (
            &["run", "game"],
            &[
                "error: file `game` does not exist",
                "help: did you mean `game.jai`?",
            ],
        ),
        (
            &["run", "nothing_like_it.jai"],
            &[
                "error: file `nothing_like_it.jai` does not exist",
                "help: relative paths start from the current directory",
            ],
        ),
        (
            &["check", "proj"],
            &[
                "error: `proj` is a directory, not a .jai file",
                "help: did you mean `proj/first.jai`?",
            ],
        ),
    ];
    for (args, expected) in cases {
        let output = jaic(&dir, args, &[]);
        assert_eq!(output.status.code(), Some(1), "{args:?}");
        let text = stderr(&output);
        assert_in_order(&text, expected);
        // No snippet of some unrelated file (the stdlib's Preload used to be shown).
        assert!(!text.contains("Preload"), "{text}");
    }
}

#[test]
fn build_problems_say_what_to_change() {
    let dir = scratch("build");
    std::fs::write(dir.join("ok.jai"), "main :: () {}\n").unwrap();
    std::fs::write(dir.join("lib.jai"), "f :: () {}\n").unwrap();
    std::fs::create_dir_all(dir.join("out")).unwrap();
    let cases: &[(&[&str], Option<i32>, &[&str])] = &[
        (
            &["build", "lib.jai"],
            Some(1),
            &[
                "error: `lib.jai` has no `main` procedure, so there is no program to write",
                "help: add `main :: () { ... }`",
            ],
        ),
        (
            &["build", "ok.jai", "-o", "out"],
            Some(1),
            &[
                "error: the output path `out` is a directory",
                "help: name the file to write inside it, as in `-o out/program`",
            ],
        ),
        (
            &["build", "ok.jai", "-sanitize", "adress"],
            Some(2),
            &[
                "error: unknown sanitizer `adress` for `-sanitize`",
                "help: use address, undefined or both",
            ],
        ),
    ];
    for (args, code, expected) in cases {
        let output = jaic(&dir, args, &[]);
        let text = stderr(&output);
        assert_eq!(output.status.code(), *code, "{args:?}: {text}");
        assert_in_order(&text, expected);
    }
    // A linker that is not installed: how to get one, and that `jaic run` needs none.
    let output = Command::new(JAIC)
        .args(["build", "ok.jai", "-o", "prog"])
        .current_dir(&dir)
        .env("JAIC_LINKER", "no-such-linker-for-jaic-tests")
        .env_remove("JAIC_DIAGNOSTICS")
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(1));
    assert_in_order(
        &stderr(&output),
        &[
            "error: could not find the linker `no-such-linker-for-jaic-tests`",
            "help: ",
            "JAIC_LINKER",
            "help: `jaic run` needs no linker",
        ],
    );
    // An undeclared `#foreign` library.
    let output = jaic_on(
        &dir,
        "f.jai",
        "f :: () #foreign nothere;\nmain :: () { f(); }\n",
        "check",
        &[],
    );
    assert_in_order(
        &stderr(&output),
        &[
            "f.jai:1:18: error: unknown library `nothere`",
            "help: declare it with `nothere :: #library",
        ],
    );
}

#[test]
fn command_line_mistakes_explain_themselves() {
    let dir = scratch("cli");
    std::fs::write(dir.join("ok.jai"), "main :: () {}\n").unwrap();
    let cases: &[(&[&str], &[&str])] = &[
        (
            &[],
            &[
                "error: no command given",
                "help: run a program with `jaic run file.jai`",
            ],
        ),
        (
            &["rnu", "ok.jai"],
            &[
                "error: unknown command `rnu`",
                "help: did you mean `jaic run`?",
            ],
        ),
        (
            &["ok.jai"],
            &[
                "error: `ok.jai` is not a command",
                "help: to run it, use `jaic run ok.jai`",
            ],
        ),
        (&["run"], &["error: `jaic run` needs a .jai file"]),
        (
            &["run", "-os", "linux", "ok.jai"],
            &[
                "error: expected a .jai file after `jaic run`, found `-os`",
                "help: put the file first",
            ],
        ),
        (
            &["run", "ok.jai", "-os"],
            &["error: `-os` needs an OS name: linux, windows, macos or wasm"],
        ),
        (
            &["run", "ok.jai", "-os", "linus"],
            &[
                "error: unknown OS `linus` for `-os`",
                "help: did you mean `-os linux`?",
            ],
        ),
        (
            &["run", "ok.jai", "-cpu", "z80"],
            &["error: unknown CPU `z80` for `-cpu`"],
        ),
        (&["run", "ok.jai", "-I"], &["error: `-I` needs a directory"]),
        (
            &["run", "ok.jai", "--timing"],
            &[
                "error: unknown option `--timing`",
                "help: did you mean `--timings`?",
            ],
        ),
        (
            &["run", "ok.jai", "-o", "x"],
            &["error: `-o` only applies to `jaic build`"],
        ),
        (
            &["build", "ok.jai", "-O7"],
            &[
                "error: unknown optimization level `-O7`",
                "help: use -O0, -O1, -O2 or -O3",
            ],
        ),
        (
            &["run", "ok.jai", "extra"],
            &[
                "error: unexpected argument `extra`",
                "help: arguments for the program go after `--`: `jaic run ok.jai -- extra`",
            ],
        ),
        (
            &["run", "ok.jai", "-plug", "X"],
            &["error: `-plug` works with `jaic check` and `jaic build`, not `jaic run`"],
        ),
        (
            &["run", "ok.jai", "--color", "sometimes"],
            &["error: unknown `--color` value `sometimes`"],
        ),
    ];
    for (args, expected) in cases {
        let output = jaic(&dir, args, &[]);
        let text = stderr(&output);
        assert_eq!(output.status.code(), Some(2), "{args:?}: {text}");
        assert_in_order(&text, expected);
        assert!(!text.contains("usage:"), "{args:?}: {text}");
    }
    for args in [&["--help"][..], &["-h"], &["help"], &["run", "--help"]] {
        let output = jaic(&dir, args, &[]);
        assert_eq!(output.status.code(), Some(0), "{args:?}");
        let text = String::from_utf8_lossy(&output.stdout);
        assert_in_order(&text, &["usage: jaic run <file.jai>", "exit status:"]);
    }
    let output = jaic(&dir, &["--version"], &[]);
    assert_eq!(output.status.code(), Some(0));
    assert!(String::from_utf8_lossy(&output.stdout).starts_with("jaic "));
    // `--color=always` reaches command-line errors too.
    let output = Command::new(JAIC)
        .args(["rnu", "--color=always"])
        .env("JAIC_DIAGNOSTICS", "plain")
        .output()
        .unwrap();
    assert!(
        stderr(&output).starts_with("\x1b[1;31merror\x1b[0m"),
        "{}",
        stderr(&output)
    );
}

#[test]
fn rich_layouts_on_request() {
    let dir = scratch("rich");
    let source =
        "#import \"Basic\";\nmain :: () {\n    counter := 1;\n    print(\"%\\n\", countr);\n}\n";
    std::fs::write(dir.join("u.jai"), source).unwrap();
    let rich = |layout: &str, color: &str| {
        let output = Command::new(JAIC)
            .args(["check", "u.jai", "--color", color])
            .current_dir(&dir)
            .env("JAIC_DIAGNOSTICS", layout)
            .output()
            .unwrap();
        stderr(&output)
    };
    let unicode = rich("unicode", "never");
    assert_in_order(
        &unicode,
        &[
            "error: unknown identifier",
            "╭─[",
            "u.jai:4:18]",
            " 3 │     counter := 1;",
            " 4 │     print(\"%\\n\", countr);",
            "   ·                  ━━━━━━ not found in this scope",
            " 5 │ }",
            "╰─",
            "help: a similar name exists: `counter`",
            " 4 ~     print(\"%\\n\", counter);",
        ],
    );
    let ascii = rich("ascii", "never");
    assert_in_order(
        &ascii,
        &[
            "  --> ",
            " 4 |     print(",
            "   |                  ^^^^^^ not found in this scope",
        ],
    );
    assert!(ascii.is_ascii(), "{ascii}");
    let colored = rich("unicode", "always");
    assert!(colored.contains("\x1b[1;31merror\x1b[0m"), "{colored}");
    // NO_COLOR is honoured when the choice is left to jaic.
    let output = Command::new(JAIC)
        .args(["check", "u.jai"])
        .current_dir(&dir)
        .env("JAIC_DIAGNOSTICS", "unicode")
        .env("FORCE_COLOR", "1")
        .env("NO_COLOR", "1")
        .output()
        .unwrap();
    assert!(!stderr(&output).contains('\x1b'));
}
