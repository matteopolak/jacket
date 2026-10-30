//! Three forms the official compiler rejects and Jacket rejects too, each with a code of its own:
//! an `ifx` of integer constants narrowed without a cast, an overloaded procedure used as a value
//! that two overloads fit, and a declaration named like a builtin type. They are errors in
//! imported modules as well (`docs/language/casts-and-conversions.md`, `procedures.md` and
//! `declarations-and-constants.md`).
use jaic::diagnostics::{Code, Diagnostic};
use jaic::sema::{Compiler, Options, VirtualFs};
use std::{path::Path, rc::Rc};

fn compile(main: &str, modules: &[(&str, &str)]) -> Result<(), Box<Diagnostic>> {
    let mut fs = VirtualFs::default();
    for (name, source) in modules {
        fs.insert(format!("/modules/{name}/module.jai"), source.as_bytes());
    }
    fs.insert("/main.jai", main.as_bytes());
    let mut options = Options::host();
    options.runtime_support = false;
    options.import_paths = vec![Path::new("/modules").to_path_buf()];
    options.preload_stdlib = Some(Path::new("/modules").to_path_buf());
    let mut compiler = Compiler::new(options, Rc::new(fs));
    compiler.compile_program(Path::new("/main.jai"))
}

/// The error of `main` alone, and the same error when the source is a module `User` that the
/// program imports.
fn rejected(code: Code, main: &str, in_module: &str) -> Diagnostic {
    let d = compile(main, &[]).expect_err("must be rejected");
    assert_eq!(d.code, code, "{d:?}");
    let in_a_module = compile(
        "#import \"User\"; main :: () { go(); }",
        &[("User", in_module)],
    )
    .expect_err("must be rejected in a module too");
    assert_eq!(in_a_module.code, code, "{in_a_module:?}");
    *d
}

// rules: cast.36
#[test]
fn an_ifx_of_constants_does_not_narrow_to_a_signed_type_without_a_cast() {
    let d = rejected(
        Code::ImplicitNarrowing,
        "f :: (b: bool) -> s32 { return ifx b then 1 else 2; }
        main :: () { f(true); }",
        "f :: (b: bool) -> s32 { return ifx b then 1 else 2; }
        go :: () { f(true); }",
    );
    assert_eq!(d.code.name(), "implicit-narrowing");
    assert_eq!(
        d.message,
        "an `ifx` of integer constants is an `s64`, which does not fit `s32` without a cast"
    );
}

#[test]
fn an_ifx_of_constants_still_fits_with_a_cast_or_a_wide_enough_type() {
    for source in [
        "f :: (b: bool) -> s32 { return cast(s32) ifx b then 1 else 2; }
        main :: () { f(true); }",
        "f :: (b: bool) -> s32 { return ifx b then cast(s32) 1 else 2; }
        main :: () { f(true); }",
        "f :: (b: bool) -> s64 { return ifx b then 1 else 2; }
        main :: () { f(true); }",
        "f :: (b: bool) -> u8 { return ifx b then 1 else 2; }
        main :: () { f(true); }",
    ] {
        compile(source, &[]).unwrap_or_else(|d| panic!("{source}: {d:?}"));
    }
}

// rules: proc.14
#[test]
fn an_overloaded_procedure_used_as_a_value_is_ambiguous() {
    let d = rejected(
        Code::AmbiguousProcedureValue,
        "over :: (a: s32) {}
        over :: (a: s64) {}
        main :: () { v := over; }",
        "over :: (a: s32) {}
        over :: (a: s64) {}
        go :: () { v := over; }",
    );
    assert_eq!(d.code.name(), "ambiguous-procedure-value");
    assert_eq!(d.message, "ambiguous overloaded procedure used as a value");
}

#[test]
fn two_overloads_that_fit_the_target_type_are_ambiguous() {
    let d = compile(
        "over :: (a: $T) {}
        over :: (a: s64) {}
        main :: () { v: (s64) -> void = over; }",
        &[],
    )
    .expect_err("must be rejected");
    assert_eq!(d.code, Code::AmbiguousProcedureValue, "{d:?}");
    assert_eq!(d.message, "2 overloads fit procedure type (s64)");
}

#[test]
fn an_overload_chosen_by_the_target_type_is_not_ambiguous() {
    compile(
        "over :: (a: s32) {}
        over :: (a: s64) {}
        main :: () { v: (s64) -> void = over; v(1); }",
        &[],
    )
    .unwrap_or_else(|d| panic!("{d:?}"));
}

// rules: decl.25
#[test]
fn a_declaration_named_like_a_builtin_type_is_an_error() {
    let d = rejected(
        Code::ShadowsBuiltinType,
        "Code :: enum s16 { A; }
        main :: () { }",
        "Code :: enum s16 { A; }
        go :: () { }",
    );
    assert_eq!(d.code.name(), "shadows-builtin-type");
    assert_eq!(
        d.message,
        "`Code` is the name of a builtin type and cannot be declared"
    );
    assert_eq!(d.help, ["choose another name"]);
}

#[test]
fn every_builtin_type_name_is_rejected_and_other_names_are_not() {
    for name in [
        "int", "float", "bool", "string", "void", "Any", "Code", "s8", "s16", "s32", "s64", "u8",
        "u16", "u32", "u64", "float32", "float64",
    ] {
        let d = compile(
            &format!("{name} :: struct {{ x: s64; }} main :: () {{ }}"),
            &[],
        )
        .expect_err(name);
        assert_eq!(d.code, Code::ShadowsBuiltinType, "{name}: {d:?}");
    }
    compile(
        "Type_Like :: struct { x: s64; }
        E :: enum { int; float; }
        main :: () { for string: 0..1 {} f :: (code: s64) {} }",
        &[],
    )
    .unwrap_or_else(|d| panic!("{d:?}"));
}
