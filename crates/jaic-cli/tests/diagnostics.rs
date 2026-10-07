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

/// A fault in C code that `jaic run` or `#run` called names the foreign procedure, the line
/// that called it and the interpreter's call stack, and exits with its own status.
#[cfg(unix)]
#[test]
fn crash_in_native_code_names_the_foreign_call() {
    let dir = scratch("native-crash");
    let source = "libc :: #system_library \"libc\";\nstrlen :: (s: *u8) -> u64 #foreign libc;\nmeasure :: (p: *u8) -> u64 {\n    return strlen(p);\n}\nwork :: () {\n    n := measure(cast(*u8) 16);\n}\n";
    let run = jaic_on(
        &dir,
        "crash.jai",
        &format!("{source}main :: () {{\n    work();\n}}\n"),
        "run",
        &[],
    );
    assert_eq!(run.status.code(), Some(121), "{}", stderr(&run));
    assert_in_order(
        &stderr(&run),
        &[
            "crash.jai:4:5: error: native code crashed (SIGSEGV, invalid memory access at address 0x10) while calling foreign procedure `strlen`",
            "note: call stack (innermost first):",
            "    `measure` at crash.jai:4",
            "    `work` at crash.jai:7",
            "    `main` at crash.jai:10",
            "help: check the arguments passed to `strlen`",
        ],
    );
    let compile_time = jaic_on(
        &dir,
        "compile_time.jai",
        &format!("{source}#run work();\nmain :: () {{}}\n"),
        "check",
        &[],
    );
    assert_eq!(compile_time.status.code(), Some(121));
    assert_in_order(
        &stderr(&compile_time),
        &[
            "compile_time.jai:4:5: error: native code crashed",
            "    `measure` at compile_time.jai:4",
            "    `work` at compile_time.jai:7",
        ],
    );
}

/// A crash on a thread C started itself is not blamed on the foreign call the interpreter is
/// making meanwhile (`pthread_join` here): no foreign call is in progress on that thread, so
/// the fault takes its default course.
#[cfg(unix)]
#[test]
fn crash_on_a_c_thread_is_not_blamed_on_another_call() {
    let dir = scratch("native-crash-thread");
    let source = "libc :: #system_library \"libc\";\npthread_create :: (thread: *u64, attributes: *void, start: *void, argument: *void) -> s32 #foreign libc;\npthread_join :: (thread: u64, result: **void) -> s32 #foreign libc;\nmain :: () {\n    thread: u64;\n    pthread_create(*thread, null, cast(*void) 16, null);\n    pthread_join(thread, null);\n}\n";
    let run = jaic_on(&dir, "thread.jai", source, "run", &[]);
    let err = stderr(&run);
    assert!(!run.status.success(), "{err}");
    assert_ne!(run.status.code(), Some(121), "{err}");
    assert!(!err.contains("native code crashed"), "{err}");
}

// rules: cast.2 flow.27
#[test]
fn cast_and_switch_checks_say_which_value() {
    let dir = scratch("value-checks");
    let cast = jaic_on(
        &dir,
        "cast.jai",
        "main :: () {\n    w := 300;\n    c := cast(u8) w;\n}\n",
        "run",
        &[],
    );
    assert_eq!(cast.status.code(), Some(1));
    assert_in_order(
        &stderr(&cast),
        &[
            "cast.jai:3:5: error: runtime error: cast of 300 to `u8` overflows",
            "    c := cast(u8) w;",
            "help: `u8` holds 0 to 255; write `cast,trunc(u8)` (or `xx,trunc`) to keep the low bits, or `cast,no_check(u8)` to skip the check",
        ],
    );
    let signed = jaic_on(
        &dir,
        "signed.jai",
        "main :: () {\n    n := -200;\n    c := cast(s8) n;\n}\n",
        "run",
        &[],
    );
    assert_in_order(
        &stderr(&signed),
        &[
            "signed.jai:3:5: error: runtime error: cast of -200 to `s8` overflows",
            "help: `s8` holds -128 to 127;",
        ],
    );
    let switch = jaic_on(
        &dir,
        "switch.jai",
        "Color :: enum { RED; GREEN; }\nmain :: () {\n    c := cast(Color) 7;\n    if #complete c == {\n        case .RED;\n        case .GREEN;\n    }\n}\n",
        "run",
        &[],
    );
    assert_eq!(switch.status.code(), Some(1));
    assert_in_order(
        &stderr(&switch),
        &[
            "switch.jai:4:5: error: runtime error: no case of the `#complete` switch matches its value, 7",
            "help: the value is not one of the enum's members",
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
    // The condition as written, commas in strings and grouping parentheses included.
    let output = jaic_on(
        &dir,
        "d.jai",
        "#import \"Basic\";\nf :: (s: string) -> int { return s.count; }\nmain :: () {\n    if true assert(f(\"a,b\") == 1 && (2 > 1));\n}\n",
        "run",
        &[],
    );
    assert_in_order(
        &stderr(&output),
        &[
            "d.jai:4:13: error: runtime error: assertion failed: `f(\"a,b\") == 1 && (2 > 1)` is false",
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
fn unknown_identifier_names_the_module_or_metaprogram_that_declares_it() {
    let dir = scratch("unknown-module-name");
    // A standard-library name without its `#import`.
    let output = jaic_on(
        &dir,
        "i.jai",
        "main :: () {\n    print(\"hi\\n\");\n}\n",
        "check",
        &[],
    );
    assert_in_order(
        &stderr(&output),
        &[
            "i.jai:2:5: error: unknown identifier `print`",
            "help: `print` is declared in the `Basic` module: add `#import \"Basic\";` to this file",
        ],
    );
    // A constant a build metaprogram adds with `add_build_string`, checked on its own.
    std::fs::create_dir_all(dir.join("src")).unwrap();
    std::fs::write(
        dir.join("build.jai"),
        "#import \"Compiler\";\n#run {\n    w := compiler_create_workspace();\n    add_build_string(\"MODE :: 2;\", w);\n    add_build_file(\"src/main.jai\", w);\n}\n",
    )
    .unwrap();
    let output = jaic_on(
        &dir,
        "src/main.jai",
        "#import \"Basic\";\nmain :: () { print(\"%\\n\", MODE); }\n",
        "check",
        &[],
    );
    assert_in_order(
        &stderr(&output),
        &[
            "error: unknown identifier `MODE`",
            "help: `MODE` is added by `build.jai` (with `add_build_string`) when it builds this file: build through it, as in `jaic build build.jai`",
        ],
    );
}

#[test]
fn mismatches_point_at_the_value_and_say_what_was_expected() {
    let dir = scratch("mismatch");
    let cases: &[(&str, &[&str])] = &[
        (
            "main :: () {\n    x: int = \"hello\";\n}\n",
            &[
                "m.jai:2:14: error: type mismatch: expected `s64`, found `string`",
                "m.jai:2:8: note: expected because of this type",
            ],
        ),
        (
            "f :: (x: int) -> string {\n    return x;\n}\nmain :: () { f(1); }\n",
            &[
                "m.jai:2:12: error: type mismatch: expected `string`, found `s64`",
                "m.jai:1:18: note: expected because of this return type",
                "help: to turn a number into text, format it",
            ],
        ),
        (
            "g :: () -> int, bool {\n    return 1, \"no\";\n}\nmain :: () { g(); }\n",
            &[
                "m.jai:2:15: error: type mismatch: expected `bool`, found `string`",
                "m.jai:1:17: note: expected because of this return type",
            ],
        ),
        (
            "main :: () { a := 1.5; b: int = a; }\n",
            &[
                "error: type mismatch: expected `s64`, found `float32`",
                "help: convert with `cast(s64)`, which drops the fraction",
            ],
        ),
        (
            "f :: (a: int, b: float) {}\nmain :: () {\n    f(1, 2.0, 3);\n}\n",
            &[
                "m.jai:3:15: error: in call to `f`: too many arguments: it takes at most 2",
                "m.jai:1:6: note: `f` is declared here",
            ],
        ),
        (
            "f :: (count: int) {}\nmain :: () { f(cuont = 1); }\n",
            &[
                "error: in call to `f`: no parameter named `cuont`",
                "help: did you mean `count`?",
            ],
        ),
        (
            "f :: (a: int, b: int) {}\nmain :: () { f(1); }\n",
            &[
                "error: in call to `f`: missing argument for parameter `b`",
                "note: `f` is declared here",
                "help: pass a value for `b`",
            ],
        ),
        (
            "P :: struct { width: int; }\nmain :: () { p: P; p.widht = 1; }\n",
            &[
                "error: type `P` has no member `widht`",
                "help: a member with a similar name exists: `width`",
            ],
        ),
        (
            "main :: () { x := 1; x := 2; }\n",
            &[
                "m.jai:1:22: error: `x` is already declared in this scope",
                "m.jai:1:14: note: `x` is first declared here",
                "help: to change its value, assign with `x = ...`",
            ],
        ),
        (
            "f :: () {}\nmain :: () { r := f(); }\n",
            &[
                "error: cannot declare `r` from `f()`, which has no value",
                "help: a procedure without a return type returns nothing",
            ],
        ),
        (
            // `n + "0"` is a byte; `*` takes no one-character string.
            "twice :: (n: u8) -> u8 { return n * \"2\"; }\nmain :: () {}\n",
            &[
                "error: type mismatch: `u8` and `string` cannot be combined",
                "help: `\"2\"` is a string; for the character's code write `#char \"2\"`",
            ],
        ),
        (
            "#assert size_of(int) == 4;\nmain :: () {}\n",
            &["error: #assert failed: `size_of(int) == 4` is false"],
        ),
    ];
    for (source, expected) in cases {
        let output = jaic_on(&dir, "m.jai", source, "check", &[]);
        assert_eq!(output.status.code(), Some(1), "{source}");
        assert_in_order(&stderr(&output), expected);
    }
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
    // A module in a folder jaic does not search: say which `-import_dir` finds it.
    std::fs::create_dir_all(dir.join("libs/Gadgets")).unwrap();
    std::fs::write(dir.join("libs/Gadgets/module.jai"), "gadget :: 1;\n").unwrap();
    let output = jaic_on(
        &dir,
        "g.jai",
        "#import \"Gadgets\";\nmain :: () {}\n",
        "check",
        &[],
    );
    assert_in_order(
        &stderr(&output),
        &[
            "help: a module `Gadgets` exists in `libs`, which is not searched: pass `-import_dir libs`",
        ],
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
            Some(3),
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
        (
            &["run", "ok.jai", "-color=sometimes"],
            &["error: unknown `--color` value `sometimes`"],
        ),
        (
            &["build", "ok.jai", "-no_workspace_output"],
            &["error: `-no_workspace_output` only applies to `jaic run`"],
        ),
        // `-plug` after `-` is the metaprogram's argument, so `-o` is still build-only.
        (
            &["run", "ok.jai", "-o", "x", "-", "-plug", "X"],
            &["error: `-o` only applies to `jaic build`"],
        ),
    ];
    for (args, expected) in cases {
        let output = jaic(&dir, args, &[]);
        let text = stderr(&output);
        assert_eq!(output.status.code(), Some(2), "{args:?}: {text}");
        assert_in_order(&text, expected);
        assert!(!text.contains("usage:"), "{args:?}: {text}");
    }
    for args in [
        &["--help"][..],
        &["-h"],
        &["help"],
        &["run", "--help"],
        &["run", "ok.jai", "--help"],
        &["build", "ok.jai", "-O2", "-h"],
    ] {
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
