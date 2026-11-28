//! Integer division and remainder as the interpreter runs them: signed and unsigned at every
//! width, `MIN / -1`, truncation toward zero, and division by zero.
use std::{path::PathBuf, rc::Rc};

use jaic::sema::{Compiler, NativeFs, Options, ProgramSource};

fn compile(source: &str) -> Result<(), String> {
    let stdlib = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../stdlib");
    let mut options = Options::host();

    options.import_paths = vec![stdlib.clone()];
    options.set_stdlib(&NativeFs, &stdlib);
    let mut c = Compiler::new(options, Rc::new(NativeFs));

    c.compile_sources(&[ProgramSource::String(source.into())])
        .map_err(|e| e.render(&c.src.sources))
}

#[test]
fn signed_division_truncates_toward_zero_and_wraps_min_over_minus_one() {
    compile(
        r"
run :: () -> s64 {
    min64 := -9223372036854775807 - 1; one64: s64 = -1;
    if min64 / one64 != min64 return 1;
    if min64 % one64 != 0 return 2;
    min32: s32 = -2147483647 - 1; one32: s32 = -1;
    if min32 / one32 != min32 return 3;
    if min32 % one32 != 0 return 4;
    min16: s16 = -32768; one16: s16 = -1;
    if min16 / one16 != min16 return 5;
    if min16 % one16 != 0 return 6;
    min8: s8 = -128; one8: s8 = -1;
    if min8 / one8 != min8 return 7;
    if min8 % one8 != 0 return 8;

    seven: s64 = 7; two: s64 = 2; neg_seven: s64 = -7; neg_two: s64 = -2;
    if neg_seven / two != -3 return 10;
    if neg_seven % two != -1 return 11;
    if seven / neg_two != -3 return 12;
    if seven % neg_two != 1 return 13;
    if neg_seven / neg_two != 3 return 14;
    if neg_seven % neg_two != -1 return 15;

    a: s8 = -100; b: s8 = 7;
    if a / b != -14 return 20;
    if a % b != -2 return 21;
    c: s16 = 30000; d: s16 = -7;
    if c / d != -4285 return 22;
    if c % d != 5 return 23;
    return 0;
}
#assert run() == 0;
main :: () {}
",
    )
    .unwrap();
}

#[test]
fn unsigned_division_uses_the_full_width() {
    compile(
        r"
run :: () -> s64 {
    big: u64 = 18446744073709551615; three: u64 = 3; ten: u64 = 10;
    if big / three != 6148914691236517205 return 1;
    if big % ten != 5 return 2;
    if big / big != 1 return 3;
    half: u64 = 9223372036854775808;
    if half / three != 3074457345618258602 return 4;
    if half % three != 2 return 5;
    w: u32 = 4294967295; sixteen: u32 = 16;
    if w / sixteen != 268435455 return 6;
    if w % sixteen != 15 return 7;
    x: u16 = 65535; y: u16 = 256;
    if x / y != 255 || x % y != 255 return 8;
    p: u8 = 255; q: u8 = 16;
    if p / q != 15 || p % q != 15 return 9;
    return 0;
}
#assert run() == 0;
main :: () {}
",
    )
    .unwrap();
}

#[test]
fn division_by_zero_is_reported_for_every_kind() {
    for (ty, op) in [
        ("s64", "/"),
        ("s64", "%"),
        ("s32", "/"),
        ("u64", "/"),
        ("u64", "%"),
        ("u8", "%"),
        ("s8", "/"),
    ] {
        let source = format!(
            "div :: (a: {ty}, b: {ty}) -> {ty} {{ return a {op} b; }}\n\
             zero :: () -> {ty} {{ return 0; }}\n\
             #run div(7, zero());\n\
             main :: () {{}}"
        );
        let error = compile(&source).expect_err("dividing by zero must stop the compile");

        assert!(
            error.to_lowercase().contains("divi") && error.to_lowercase().contains("zero"),
            "{ty} {op}: {error}"
        );
    }
}
