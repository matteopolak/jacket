//! Programs shaped like the code that modules written for other Jai installs contain: each is
//! type-checked with `-check`, with and without `-no_dce`, and must be accepted.
use std::{path::PathBuf, process::Command};

/// Writes `source` as `<name>/main.jai` under the test scratch folder and runs
/// `jaic -check <extra args>` on it; returns the failure text, or `None` when it checks.
fn check_with(name: &str, source: &str, extra: &[&str]) -> Option<String> {
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
        .join("module-compat")
        .join(name);
    std::fs::create_dir_all(&dir).unwrap();
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
