//! Runtime failures name the check and the call stack.
use super::{assert_in_order, jaic_on, scratch, stderr};

#[test]
fn runtime_bounds_check_names_the_check_and_the_call_stack() {
    let dir = scratch("bounds");
    let output = jaic_on(
        &dir,
        "bounds.jai",
        "get :: (a: [] int, i: int) -> int {\n    return a[i];\n}\nmain :: () {\n    arr := int.[1, 2, 3];\n    get(arr, 5);\n}\n",
        "-interp",
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
        "-interp",
        &[],
    );
    assert_eq!(output.status.code(), Some(1));
    assert_in_order(
        &stderr(&output),
        &[
            "copy.jai:5:5: error: runtime error: null pointer dereference: memcpy through a null pointer",
            "    t := copy_string(s);",
            "note: this call failed inside `copy_string` at ",
            "allocation.jai",
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
        "-interp",
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
        "-check",
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

/// A failed check in a procedure C called ends the run (it cannot unwind through C), saying
/// where it failed and which procedures it was in.
#[cfg(unix)]
#[test]
fn failure_in_a_procedure_called_from_c_says_where() {
    let dir = scratch("callback-trap");
    let source = "libc :: #system_library \"libc\";\nqsort :: (base: *void, n: u64, size: u64, compare: (*void, *void) -> s32 #c_call) #foreign libc;\ncompare :: (a: *void, b: *void) -> s32 #c_call {\n    x: [2] s32;\n    i := 5;\n    return x[i];\n}\nmain :: () {\n    a := int.[3, 1, 2];\n    qsort(a.data, 3, 8, compare);\n}\n";
    let run = jaic_on(&dir, "callback.jai", source, "-interp", &[]);
    assert_eq!(run.status.code(), Some(1), "{}", stderr(&run));
    assert_in_order(
        &stderr(&run),
        &[
            "error: in a procedure called from C: array bounds check failed: index 5 is outside an array of 2 elements (at callback.jai:6:5)",
            "    in `compare` at callback.jai:6:5",
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
    let run = jaic_on(&dir, "thread.jai", source, "-interp", &[]);
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
        "-interp",
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
        "-interp",
        &[],
    );
    assert_in_order(
        &stderr(&signed),
        &[
            "signed.jai:3:5: error: runtime error: cast of -200 to `s8` overflows",
            "help: `s8` holds -128 to 127;",
        ],
    );
    // A change of sign is checked at the same width, and `xx` is checked like `cast`.
    for (name, body, message) in [
        (
            "sign.jai",
            "n := -1;\n    c := cast(u64) n;",
            "cast of -1 to `u64` overflows",
        ),
        (
            "autocast.jai",
            "n := 70000;\n    c: s16 = xx n;",
            "cast of 70000 to `s16` overflows",
        ),
    ] {
        let run = jaic_on(
            &dir,
            name,
            &format!("main :: () {{\n    {body}\n}}\n"),
            "-interp",
            &[],
        );
        assert_eq!(run.status.code(), Some(1));
        assert!(stderr(&run).contains(message), "{}", stderr(&run));
    }
    let switch = jaic_on(
        &dir,
        "switch.jai",
        "Color :: enum { RED; GREEN; }\nmain :: () {\n    c := cast(Color) 7;\n    if #complete c == {\n        case .RED;\n        case .GREEN;\n    }\n}\n",
        "-interp",
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
        "-interp",
        &[],
    );
    assert_in_order(
        &stderr(&null),
        &[
            "null.jai:4:5: error: runtime error: null pointer dereference: read through a null pointer",
        ],
    );
    // Boxing an lvalue records its address/type without loading it. Test the
    // compiler representation directly; reflected printing has its own contract.
    let boxed = jaic_on(
        &dir,
        "boxed.jai",
        "capture :: (value: Any) -> s32 { if value.value_pointer == null && value.type == type_info(int) return 0; return 1; }\nmain :: () -> s32 { p: *int; return capture(p.*); }\n",
        "-interp",
        &[],
    );
    assert_eq!(boxed.status.code(), Some(0), "{}", stderr(&boxed));
    assert!(stderr(&boxed).is_empty(), "{}", stderr(&boxed));
    assert!(boxed.stdout.is_empty());
    let missing = jaic_on(
        &dir,
        "sign.jai",
        "sign :: (x: int) -> int {\n    if x > 0 return 1;\n}\nmain :: () {\n    sign(0);\n}\n",
        "-interp",
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
        "-interp",
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
        "-check",
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
        "-interp",
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
        "-interp",
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
        "-interp",
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
        "-interp",
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
        "-check",
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
