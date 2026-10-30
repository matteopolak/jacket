//! Programs shaped like the code that modules written for other Jai installs contain: each is
//! type-checked with `-check`, with and without `-no_dce`, and must be accepted.
use std::process::Command;
use test_support::TempDir;

/// Writes `source` as `<name>/main.jai` under the test scratch folder and runs
/// `jaic -check <extra args>` on it; returns the failure text, or `None` when it checks.
fn check_with(name: &str, source: &str, extra: &[&str]) -> Option<String> {
    let dir = TempDir::new(name);
    let file = dir.join("main.jai");
    std::fs::write(&file, source).unwrap();
    let mut command = Command::new(env!("CARGO_BIN_EXE_jaic"));
    command.arg("-check").arg(&file).args(extra);
    let output = command.output().unwrap();
    if output.status.success() {
        None
    } else {
        Some(String::from_utf8_lossy(&output.stderr).into_owned())
    }
}

/// Writes `files` (path, source) under the test scratch folder `name`, with `-I` on that folder,
/// and asserts that its `main.jai` checks with and without `-no_dce`.
fn accepted_with_modules(name: &str, files: &[(&str, &str)]) {
    let dir = TempDir::new(name);
    for (path, source) in files {
        let file = dir.join(path);
        std::fs::create_dir_all(file.parent().unwrap()).unwrap();
        std::fs::write(file, source).unwrap();
    }
    for extra in [&[][..], &["-no_dce"][..]] {
        let output = Command::new(env!("CARGO_BIN_EXE_jaic"))
            .arg("-check")
            .arg(dir.join("main.jai"))
            .arg("-I")
            .arg(&dir)
            .args(extra)
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{name} {extra:?} was rejected:\n{}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
}

/// Asserts that `source` checks with and without `-no_dce`.
fn accepted(name: &str, source: &str) {
    for extra in [&[][..], &["-no_dce"][..]] {
        if let Some(failure) = check_with(name, source, extra) {
            panic!("{name} {extra:?} was rejected:\n{failure}");
        }
    }
}

#[test]
fn stack_trace_constant_is_predeclared() {
    accepted(
        "stack-trace-constant",
        r#"#import "Basic";
Holder :: struct {
    value: int;
    #if _STACK_TRACE  sentinel: Stack_Trace_Node;
}
main :: () {
    h: Holder;
    #if _STACK_TRACE  h.sentinel.call_depth = 1;
    print("%\n", h.value);
}
"#,
    );
}

#[test]
fn autocast_mask_operand_of_a_logical_operator_keeps_its_integer_type() {
    accepted(
        "autocast-mask-logical",
        r#"#import "Basic";
ANISOTROPIC :: 0x40;
Filter :: enum u32 { PLAIN :: 0; ANISOTROPIC_LINEAR :: 0x55; }

is_anisotropic :: (filter: Filter) -> bool {
    return ((xx filter) & ANISOTROPIC) && filter != .PLAIN;
}

main :: () {
    assert(is_anisotropic(.ANISOTROPIC_LINEAR));
    assert(!is_anisotropic(.PLAIN));
}
"#,
    );
}

#[test]
fn procedure_names_compare_with_null() {
    accepted(
        "procedure-null",
        r#"#import "Basic";
local :: () {}
libc :: #system_library "libc";
puts :: (text: *u8) -> s32 #foreign libc;
main :: () {
    assert(local != null);
    assert(null != local);
    assert(puts != null);
    assert(!(local == null));
    loaded := local != null;
    if puts == null  return;
}
"#,
    );
}

#[test]
fn complement_of_a_constant_converts_to_a_narrow_integer_with_xx() {
    accepted(
        "complement-xx",
        r#"#import "Basic";
NONBLOCK :: 0x4;
CLOSE_ON_EXEC : u32 : 0x1;
main :: () {
    flags: s32 = 7;
    flags &= xx ~NONBLOCK;
    assert(flags == 3);
    narrow: s16 = xx ~NONBLOCK;
    assert(narrow == -5);
    low: u32 = xx ~CLOSE_ON_EXEC;
    assert(low == 0xffff_fffe);
    wide: s32 = cast(s32) ~NONBLOCK;
    assert(wide == -5);
}
"#,
    );
}

#[test]
fn procedure_literal_field_may_return_several_values() {
    accepted(
        "field-multi-return-literal",
        r#"#import "Basic";
Pad :: struct { id: int; }
Handler :: struct {
    proc: (data: *u8) -> bool, Pad;
    other: int;
}
make :: () -> Handler {
    return Handler.{
        proc = (data: *u8) -> bool, Pad {
            return true, .{1};
        },
        other = 3,
    };
}
main :: () {
    h := make();
    ok, pad := h.proc(null);
    assert(ok && pad.id == 1 && h.other == 3);
}
"#,
    );
}

#[test]
fn own_procedure_wins_over_an_imported_one_that_fits_alike() {
    accepted_with_modules(
        "own-overload-beats-import",
        &[
            (
                "main.jai",
                "Mod :: #import \"Mod\";\nmain :: () { a, b := Mod.take(); }\n",
            ),
            (
                "Mod/module.jai",
                r#"#import "Sys";
grab :: (size: s64, flags: s32) -> *void, s64 { return null, 0; }

take :: () -> *void, s64 {
    ptr: *void;
    count: s64;
    ptr, count = grab(8, 3);
    return ptr, count;
}
"#,
            ),
            (
                "Sys/module.jai",
                "grab :: (size: s64, flags: s32) -> s32 { return 0; }\n",
            ),
        ],
    );
}

#[test]
fn imported_procedure_a_body_uses_wins_over_the_modules_own() {
    accepted_with_modules(
        "using-beats-own-overload",
        &[
            (
                "main.jai",
                "Mod :: #import \"Mod\";\nmain :: () { a := Mod.take(); }\n",
            ),
            (
                "Mod/module.jai",
                r#"#import "Sys";
grab :: (size: s64, flags: s32) -> *void, s64 { return null, 0; }

take :: () -> s32 {
    S :: #import "Sys";
    using S;
    n: s32 = grab(8, 3);
    return n;
}
"#,
            ),
            (
                "Sys/module.jai",
                "grab :: (size: s64, flags: s32) -> s32 { return 0; }\n",
            ),
        ],
    );
}

// rules: scope.18
#[test]
fn variable_a_module_declares_beats_a_procedure_its_file_imports() {
    accepted_with_modules(
        "own-variable-beats-import",
        &[
            (
                "main.jai",
                "#import \"Holder\";\nmain :: () { set_it(); }\n",
            ),
            ("Sys/module.jai", "errno :: () -> s32 { return 0; }\n"),
            ("Holder/module.jai", "#load \"a.jai\";\n#load \"b.jai\";\n"),
            ("Holder/a.jai", "errno: s32;\n"),
            (
                "Holder/b.jai",
                "set_it :: () {\n    errno = 5;\n}\n\n#scope_file\n#import \"Sys\";\n",
            ),
        ],
    );
}

#[test]
fn overlay_on_a_missing_member_path_is_an_error() {
    let text = check_with(
        "overlay-missing-member",
        "Header :: struct { tag: u32; }\nObject :: struct {\n    base: Header;\n    #overlay(base.table) typed: *u32;\n}\nmain :: () { o: Object; }\n",
        &[],
    )
    .expect("an overlay on a member that does not exist must be rejected");
    assert!(
        text.contains("no member named `table` to anchor on"),
        "{text}"
    );
}
