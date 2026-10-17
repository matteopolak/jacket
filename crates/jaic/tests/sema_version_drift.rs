//! Checks that follow the current official compiler: array equality, constant division by
//! zero, `ifx` branch agreement, one-character strings as bytes, and the type of a
//! complemented constant next to an unsigned operand.
use jaic::sema::{Compiler, Options, VirtualFs};
use std::{path::Path, rc::Rc};

fn compile(source: &str) -> Result<(), String> {
    let mut fs = VirtualFs::default();
    fs.insert("/main.jai", source.as_bytes());
    let mut options = Options::host();
    options.runtime_support = false;
    let mut compiler = Compiler::new(options, Rc::new(fs));
    compiler
        .compile_program(Path::new("/main.jai"))
        .map_err(|e| e.render(&compiler.sources))
}

/// Wraps statements in a procedure whose result is asserted at compile time.
fn checked(body: &str) -> String {
    format!("check :: () -> bool {{ {body} }}\n#assert check();\nmain :: () {{}}\n")
}

fn accepts(name: &str, body: &str) {
    if let Err(e) = compile(&checked(body)) {
        panic!("{name}: expected to compile, got:\n{e}");
    }
}

/// `source` must fail with `message` reported at the 1-based `line:col`.
fn rejects(name: &str, source: &str, line: usize, col: usize, message: &str) {
    let e = compile(source).expect_err(name);
    let at = format!("main.jai:{line}:{col}: error: ");
    assert!(e.contains(&at), "{name}: no error at {line}:{col}:\n{e}");
    assert!(e.contains(message), "{name}: missing `{message}`:\n{e}");
}

#[test]
fn arrays_do_not_compare() {
    for (name, decl) in [
        ("fixed", "a: [3]int; b: [3]int;"),
        ("slice", "a: []int; b: []int;"),
        ("dynamic", "a: [..]int; b: [..]int;"),
    ] {
        for op in ["==", "!="] {
            let prefix = format!("f :: () {{ {decl} r := ");
            let src = format!("{prefix}a {op} b; }}\nmain :: () {{}}\n");
            rejects(
                &format!("{name} {op}"),
                &src,
                1,
                prefix.len() + 1,
                &format!("arrays cannot be compared with `{op}`"),
            );
        }
    }
    accepts("strings compare", r#"return "abc" == "abc" && "a" != "b";"#);
}

#[test]
fn constant_division_by_zero_is_an_error() {
    for (name, src) in [
        ("float/float", "X :: 1.0 / 0.0;\nmain :: () {}\n"),
        ("zero/zero", "X :: 0.0 / 0.0;\nmain :: () {}\n"),
        ("float/int", "X :: 1.0 / 0;\nmain :: () {}\n"),
        ("int/int", "X :: 1 / 0;\nmain :: () {}\n"),
        (
            "named",
            "f :: () { X :: 2.0; Y :: X / 0.0; z := Y; }\nmain :: () {}\n",
        ),
        (
            "return",
            "f :: () -> float { return 1.0 / 0.0; }\nmain :: () {}\n",
        ),
    ] {
        let e = compile(src).expect_err(name);
        assert!(e.contains("division by zero"), "{name}:\n{e}");
    }
    rejects(
        "span",
        "X :: 1.0 / 0.0;\nmain :: () {}\n",
        1,
        6,
        "division by zero in constant expression",
    );
    accepts("runtime", "z := 0.0; r := 1.0 / z; return r > 1000000.0;");
}

#[test]
fn ifx_branches_must_agree() {
    for (name, expr) in [
        ("constant true", r#"ifx true then 1 else "a""#),
        ("constant false", r#"ifx false then "a" else 1"#),
        ("runtime", r#"ifx c then 1 else "a""#),
    ] {
        let src = format!("f :: (c: bool) {{ v := {expr}; }}\nmain :: () {{}}\n");
        let e = compile(&src).expect_err(name);
        assert!(e.contains("main.jai:1:"), "{name}:\n{e}");
        assert!(
            e.contains("different types") || e.contains("type mismatch"),
            "{name}:\n{e}"
        );
    }
    rejects(
        "notes",
        "f :: () { v := ifx true then 1 else \"a\"; }\nmain :: () {}\n",
        1,
        16,
        "the branches of `ifx` have different types",
    );
    let e = compile("f :: () { v := ifx true then 1 else \"a\"; }\nmain :: () {}\n").unwrap_err();
    assert!(
        e.contains("the `then` value is") && e.contains("the `else` value is `string`"),
        "{e}"
    );
    accepts(
        "same type",
        r#"s := ifx false then "b" else "a"; return s == "a";"#,
    );
    accepts(
        "numeric widening",
        "a: u8 = 1; b: s32 = 2; r := ifx true then a else b; return r == 1;",
    );
    rejects(
        "string to u8 through ifx",
        "f :: () { c: u8 = ifx true then \"b\" else \"a\"; }\nmain :: () {}\n",
        1,
        19,
        "expected `u8`, found `string`",
    );
}

#[test]
fn one_character_strings_are_bytes_only_for_u8() {
    accepts("declaration", r#"x: u8 = "a"; return x == 97;"#);
    accepts("store", r#"b: [4]u8; b[0] = "z"; return b[0] == 122;"#);
    accepts("compound", r#"x: u8 = 1; x += "a"; return x == 98;"#);
    accepts("binary", r#"x: u8 = 1; x = x + "a"; return x == 98;"#);
    accepts("compare", r#"x: u8 = 97; return x == "a";"#);
    for ty in ["u16", "int", "s8"] {
        let prefix = format!("f :: () {{ x: {ty} = 1; ");
        let src = format!("{prefix}x += \"a\"; }}\nmain :: () {{}}\n");
        rejects(ty, &src, 1, prefix.len() + 1, "cannot be combined");
    }
    let e = compile("f :: () { x: u8 = \"ab\"; }\nmain :: () {}\n").unwrap_err();
    assert!(
        e.contains("only a one-character string converts to a `u8`"),
        "{e}"
    );
}

#[test]
fn complemented_constants_promote_next_to_unsigned() {
    let cases = [
        (
            "u32 & ~7",
            "x: u32 = 13; y := x & ~0x7; return type_of(y) == s64 && y == 8;",
        ),
        (
            "u32 | ~7",
            "x: u32 = 13; y := x | ~0x7; return type_of(y) == s64 && y == -3;",
        ),
        (
            "u32 & -8",
            "x: u32 = 13; y := x & -8; return type_of(y) == s64 && y == 8;",
        ),
        (
            "u32 + -1",
            "x: u32 = 13; y := x + -1; return type_of(y) == s64 && y == 12;",
        ),
        (
            "u32 ^ ~0",
            "x: u32 = 13; y := x ^ ~0; return type_of(y) == s64 && y == -14;",
        ),
        (
            "u32 & hex",
            "x: u32 = 13; y := x & 0xffff_fff8; return type_of(y) == u32 && y == 8;",
        ),
        (
            "u32 high",
            "x: u32 = 0xffff_fff0; y := x & ~0x7; return type_of(y) == s64 && y == 4294967280;",
        ),
        (
            "u8 & ~7",
            "x: u8 = 13; y := x & ~0x7; return type_of(y) == s64 && y == 8;",
        ),
        (
            "u16 typed",
            "x: u16 = 13; y := x & ~cast(u16) 7; return type_of(y) == u16 && y == 8;",
        ),
        (
            "s32 & ~7",
            "x: s32 = 13; y := x & ~0x7; return type_of(y) == s64 && y == 8;",
        ),
        (
            "u64 & ~7",
            "x: u64 = 13; y := x & ~0x7; return type_of(y) == u64 && y == 8;",
        ),
        ("declared", "y: u32 = ~0; return y == 0xffff_ffff;"),
    ];
    for (name, body) in cases {
        accepts(name, body);
    }
    for (name, body) in [
        ("declared u32", "x: u32 = 13; y: u32 = x & ~0x7;"),
        ("compound u32", "x: u32 = 13; x &= ~0x7;"),
    ] {
        let e = compile(&format!("f :: () {{ {body} }}\nmain :: () {{}}\n")).expect_err(name);
        assert!(e.contains("expected `u32`, found `s64`"), "{name}:\n{e}");
    }
    rejects(
        "u64 | -1",
        "f :: () { x: u64 = 13; y := x | -1; }\nmain :: () {}\n",
        1,
        29,
        "does not fit in `u64`",
    );
}
