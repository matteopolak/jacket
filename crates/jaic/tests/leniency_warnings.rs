//! Each construct Jacket accepts although the official compiler rejects it is a warning with a
//! code of its own and the shared note (`docs/language/leniencies.md`); the same construct in an
//! imported module is not reported.
use jaic::leniency::{Leniency, NOTE};
use jaic::sema::{Compiler, Options, VirtualFs};
use jaic::source::Diagnostic;
use std::{path::Path, rc::Rc};

/// The types the compiler wants from the preload file: a dynamic array carries an `Allocator`
/// and `#caller_location` has a `Source_Code_Location`.
const PRELUDE: &str = "Allocator :: struct { proc: *void; data: *void; }
Source_Code_Location :: struct { fully_pathed_filename: string; line_number: s64; character_number: s64; }";

/// A compile of `main` (`/main.jai`) with `modules` (name, source) importable.
fn compile(main: &str, modules: &[(&str, &str)]) -> (Result<(), Box<Diagnostic>>, Compiler) {
    let mut fs = VirtualFs::default();
    for (name, source) in modules {
        fs.insert(format!("/modules/{name}/module.jai"), source.as_bytes());
    }
    fs.insert("/prelude.jai", PRELUDE.as_bytes());
    fs.insert("/main.jai", main.as_bytes());
    let mut options = Options::host();
    options.runtime_support = false;
    options.preload = Some(Path::new("/prelude.jai").to_path_buf());
    options.import_paths = vec![Path::new("/modules").to_path_buf()];
    options.preload_stdlib = Some(Path::new("/modules").to_path_buf());
    let mut compiler = Compiler::new(options, Rc::new(fs));
    let result = compiler.compile_program(Path::new("/main.jai"));
    (result, compiler)
}

/// The warnings with `leniency`'s code of a program that must compile.
fn warnings(main: &str, modules: &[(&str, &str)], leniency: Leniency) -> Vec<Diagnostic> {
    let (result, compiler) = compile(main, modules);
    if let Err(error) = result {
        panic!("{}", error.render(&compiler.src.sources));
    }
    compiler
        .diag
        .warnings
        .iter()
        .filter(|w| w.kind.code() == Some(leniency.code()))
        .cloned()
        .collect()
}

/// The one warning of `leniency`, with its code, the shared note, and the text it renders to.
fn the_warning(main: &str, modules: &[(&str, &str)], leniency: Leniency) -> String {
    let (result, compiler) = compile(main, modules);
    if let Err(error) = result {
        panic!("{}", error.render(&compiler.src.sources));
    }
    let found: Vec<_> = compiler
        .diag
        .warnings
        .iter()
        .filter(|w| w.kind.code() == Some(leniency.code()))
        .collect();
    assert_eq!(found.len(), 1, "{:?}", compiler.diag.warnings);
    assert!(found[0].notes.iter().any(|(_, note)| note == NOTE));
    compiler.render(found[0])
}

const COLL: &str = "Coll :: struct { x: int; }
For_Flags :: u32;
for_expansion :: (c: *Coll, body: Code, flags: For_Flags) #expand {
    `it := 1;
    `it_index := 0;
    #insert body;
}";

#[test]
fn for_over_a_type_reached_through_a_namespace_warns() {
    let text = the_warning(
        "M :: #import \"Coll\";
        main :: () { c: M.Coll; for c {} }",
        &[("Coll", COLL)],
        Leniency::NamespacedForExpansion,
    );
    assert!(
        text.contains("`for_expansion` is not in scope here: using the one declared with `Coll`"),
        "{text}"
    );
    assert!(text.contains("add `using M;` to this scope"), "{text}");
    assert!(
        text.contains("the official Jai compiler rejects this"),
        "{text}"
    );
}

#[test]
fn for_with_the_module_imported_by_name_does_not_warn() {
    for header in ["#import \"Coll\";", "M :: #import \"Coll\"; using M;"] {
        let found = warnings(
            &format!("{header} main :: () {{ c: Coll; for c {{}} }}"),
            &[("Coll", COLL)],
            Leniency::NamespacedForExpansion,
        );
        assert!(found.is_empty(), "{header}: {found:?}");
    }
}

#[test]
fn for_inside_an_imported_module_is_silent() {
    let user = "M :: #import \"Coll\";
        go :: () { c: M.Coll; for c {} }";
    let found = warnings(
        "#import \"User\"; main :: () { go(); }",
        &[("Coll", COLL), ("User", user)],
        Leniency::NamespacedForExpansion,
    );
    assert!(found.is_empty(), "{found:?}");
}

/// A program of `main` that must warn once with `leniency`, rendered; and the same text as the
/// module `User` imported by a program of its own, which must not.
fn warns_in_the_program_and_not_in_a_module(
    leniency: Leniency,
    main: &str,
    in_module: &str,
) -> String {
    let text = the_warning(main, &[], leniency);
    let found = warnings(
        "#import \"User\"; main :: () { go(); }",
        &[("User", in_module)],
        leniency,
    );
    assert!(found.is_empty(), "{leniency:?} in a module: {found:?}");
    text
}

#[test]
fn place_warns_and_names_overlay() {
    let text = warns_in_the_program_and_not_in_a_module(
        Leniency::PlaceDirective,
        "Ts :: struct { a: u16; #place a; b: float64; }
        main :: () { t: Ts; t.b = 1; }",
        "Ts :: struct { a: u16; #place a; b: float64; }
        go :: () { t: Ts; t.b = 1; }",
    );
    assert!(
        text.contains("`#place` is a deprecated directive"),
        "{text}"
    );
    assert!(text.contains("write `#overlay(member)`"), "{text}");
}

#[test]
fn foreign_library_warns_and_names_library() {
    let text = warns_in_the_program_and_not_in_a_module(
        Leniency::ForeignLibrary,
        "m :: #foreign_library \"libm\";
        main :: () { }",
        "m :: #foreign_library \"libm\";
        go :: () { }",
    );
    assert!(
        text.contains("`#foreign_library` is not a directive here"),
        "{text}"
    );
    assert!(text.contains("write `#library`"), "{text}");
    let text = the_warning(
        "m :: #foreign_system_library \"libm\";
        main :: () { }",
        &[],
        Leniency::ForeignLibrary,
    );
    assert!(text.contains("write `#system_library`"), "{text}");
}

#[test]
fn an_unnamed_library_statement_warns() {
    let text = warns_in_the_program_and_not_in_a_module(
        Leniency::UnnamedLibrary,
        "#library,link_always \"libm\";
        main :: () { }",
        "#library,link_always \"libm\";
        go :: () { }",
    );
    assert!(text.contains("`#library` needs a name here"), "{text}");
}

#[test]
fn must_after_a_return_type_warns() {
    let text = warns_in_the_program_and_not_in_a_module(
        Leniency::MustAfterReturnType,
        "f :: () -> int #must { return 1; }
        main :: () { x := f(); }",
        "f :: () -> int #must { return 1; }
        go :: () { x := f(); }",
    );
    assert!(
        text.contains("`#must` after a return type is not a directive here"),
        "{text}"
    );
}

#[test]
fn a_semicolon_before_else_in_a_static_ifx_warns() {
    let text = warns_in_the_program_and_not_in_a_module(
        Leniency::IfxThenSemicolon,
        "main :: () { n := #ifx true then 1; else 2; }",
        "go :: () { n := #ifx true then 1; else 2; }",
    );
    assert!(text.contains("`;` before `else` ends the `#ifx`"), "{text}");
    assert!(text.contains("remove the `;`"), "{text}");
}

#[test]
fn the_address_of_a_struct_literal_warns() {
    let text = warns_in_the_program_and_not_in_a_module(
        Leniency::AddressOfLiteral,
        "T :: struct { a: int; }
        main :: () { p := *T.{1}; }",
        "T :: struct { a: int; }
        go :: () { p := *T.{1}; }",
    );
    assert!(
        text.contains("`*T.{1}` reads as a pointer type followed by a literal"),
        "{text}"
    );
    assert!(text.contains("write `*(T.{1})`"), "{text}");
}

#[test]
fn the_portable_spellings_do_not_warn() {
    for leniency in [
        Leniency::PlaceDirective,
        Leniency::ForeignLibrary,
        Leniency::UnnamedLibrary,
        Leniency::MustAfterReturnType,
        Leniency::IfxThenSemicolon,
        Leniency::AddressOfLiteral,
    ] {
        let found = warnings(
            "T :: struct { a: int; b: int; #overlay(a) c: u8; }
            m :: #library \"libm\";
            n :: #system_library \"libm\";
            f :: () -> int { return 1; }
            main :: () { p := *(T.{1, 2}); q := #ifx true then 1 else 2; r := f(); }",
            &[],
            leniency,
        );
        assert!(found.is_empty(), "{leniency:?}: {found:?}");
    }
}

#[test]
fn an_enum_loose_type_warns_and_names_the_integer_type() {
    let text = warns_in_the_program_and_not_in_a_module(
        Leniency::LooseEnum,
        "D :: enum { E; N; }
        main :: () { y: D.loose; y = 3; }",
        "D :: enum { E; N; }
        go :: () { y: D.loose; y = 3; }",
    );
    assert!(text.contains("`loose` is not a member of `D`"), "{text}");
    assert!(
        text.contains("write the integer type, `s64`, and cast to `D`"),
        "{text}"
    );
}

const SYS: &str = "errno :: () -> s32 { return 0; }
grab :: (size: s64, flags: s32) -> s32 { return 0; }
Shared :: struct { n: int; }";

#[test]
fn a_procedure_with_the_parameters_of_an_imported_one_warns() {
    let text = the_warning(
        "#import \"Sys\";
        grab :: (size: s64, flags: s32) -> *void, s64 { return null, 0; }
        main :: () { }",
        &[("Sys", SYS)],
        Leniency::HidesImport,
    );
    assert!(
        text.contains("`grab` has the same parameters as the one `Sys` exports"),
        "{text}"
    );
    assert!(text.contains("also declared here"), "{text}");
    assert!(text.contains("`M :: #import \"Sys\";`"), "{text}");
}

#[test]
fn a_name_that_is_not_a_procedure_warns_when_an_import_exports_it() {
    for declaration in ["errno: s32;", "Shared :: struct { m: int; }"] {
        let main = format!("#import \"Sys\"; {declaration} main :: () {{ }}");
        let text = the_warning(&main, &[("Sys", SYS)], Leniency::HidesImport);
        assert!(
            text.contains("is also exported by `Sys`, which this program imports"),
            "{declaration}: {text}"
        );
    }
}

#[test]
fn overloads_that_differ_and_namespaced_imports_do_not_warn() {
    for main in [
        "#import \"Sys\"; grab :: (size: s64) { } main :: () { }",
        "S :: #import \"Sys\"; grab :: (size: s64, flags: s32) { } errno: s32; main :: () { }",
        "#import \"Sys\"; main :: () { errno: s32; }",
    ] {
        let found = warnings(main, &[("Sys", SYS)], Leniency::HidesImport);
        assert!(found.is_empty(), "{main}: {found:?}");
    }
}

#[test]
fn a_declaration_of_an_imported_module_is_silent() {
    let user = "#import \"Sys\";
        errno: s32;
        grab :: (size: s64, flags: s32) -> *void, s64 { return null, 0; }
        go :: () { errno = 1; a, b := grab(1, 2); }";
    let found = warnings(
        "#import \"User\"; main :: () { go(); }",
        &[("Sys", SYS), ("User", user)],
        Leniency::HidesImport,
    );
    assert!(found.is_empty(), "{found:?}");
}

#[test]
fn a_float_exponent_without_a_fraction_warns() {
    let text = warns_in_the_program_and_not_in_a_module(
        Leniency::FloatExponent,
        "main :: () { x := 1e20; }",
        "go :: () { x := 1e20; }",
    );
    assert!(
        text.contains("`1e20` has an exponent and no fraction"),
        "{text}"
    );
    assert!(text.contains("write `1.0e20`"), "{text}");
    let found = warnings(
        "main :: () { x := 1.0e20; y := 1.5e3; }",
        &[],
        Leniency::FloatExponent,
    );
    assert!(found.is_empty(), "{found:?}");
}

#[test]
fn align_on_a_whole_struct_warns() {
    let text = warns_in_the_program_and_not_in_a_module(
        Leniency::StructAlign,
        "S :: struct #align 64 { v: s64; }
        main :: () { s: S; }",
        "S :: struct #align 64 { v: s64; }
        go :: () { s: S; }",
    );
    assert!(
        text.contains("`#align` is not a directive of a struct as a whole"),
        "{text}"
    );
    assert!(text.contains("member: T #align N;"), "{text}");
    let found = warnings(
        "S :: struct { v: s64 #align 64; }
        main :: () { s: S; }",
        &[],
        Leniency::StructAlign,
    );
    assert!(found.is_empty(), "{found:?}");
}

#[test]
fn for_over_an_array_literal_without_a_type_warns() {
    let text = warns_in_the_program_and_not_in_a_module(
        Leniency::ForOverArrayLiteral,
        "main :: () { for x: .[1, 2, 3] {} }",
        "go :: () { for x: .[1, 2, 3] {} }",
    );
    assert!(
        text.contains("`for` over an array literal needs its element type"),
        "{text}"
    );
    assert!(text.contains("for x: T.[..]"), "{text}");
    let found = warnings(
        "main :: () { for x: s64.[1, 2, 3] {} }",
        &[],
        Leniency::ForOverArrayLiteral,
    );
    assert!(found.is_empty(), "{found:?}");
}

#[test]
fn a_struct_literal_for_a_dynamic_array_warns() {
    let text = warns_in_the_program_and_not_in_a_module(
        Leniency::ArrayFromStructLiteral,
        "main :: () { a: [..] s64; a = .{}; }",
        "go :: () { a: [..] s64; a = .{}; }",
    );
    assert!(
        text.contains("an untyped `.{..}` literal is not a dynamic array"),
        "{text}"
    );
    assert!(text.contains("name the type"), "{text}");
    let found = warnings(
        "main :: () { a: [..] s64; a = type_of(a).{}; }",
        &[],
        Leniency::ArrayFromStructLiteral,
    );
    assert!(found.is_empty(), "{found:?}");
}

#[test]
fn target_features_warns() {
    let text = warns_in_the_program_and_not_in_a_module(
        Leniency::TargetFeatures,
        "f :: () -> s64 #target_features \"avx2\" { return 1; }
        main :: () { f(); }",
        "f :: () -> s64 #target_features \"avx2\" { return 1; }
        go :: () { f(); }",
    );
    assert!(
        text.contains("`#target_features` is not a directive of a procedure header"),
        "{text}"
    );
    assert!(text.contains("there is no portable spelling"), "{text}");
}

#[test]
fn push_context_without_a_block_warns() {
    let text = warns_in_the_program_and_not_in_a_module(
        Leniency::PushContextStatement,
        "run :: () {}
        main :: () { c := context; push_context c run(); }",
        "run :: () {}
        go :: () { c := context; push_context c run(); }",
    );
    assert!(text.contains("`push_context` takes a block"), "{text}");
    assert!(text.contains("push_context c { statement; }"), "{text}");
    let found = warnings(
        "run :: () {}
        main :: () { c := context; push_context c { run(); } }",
        &[],
        Leniency::PushContextStatement,
    );
    assert!(found.is_empty(), "{found:?}");
}

#[test]
fn a_main_that_returns_a_value_warns() {
    let text = the_warning(
        "main :: () -> s64 { return 0; }",
        &[],
        Leniency::MainReturnsValue,
    );
    assert!(text.contains("`main` returns a value"), "{text}");
    assert!(text.contains("exit(code)"), "{text}");
    let found = warnings(
        "#import \"User\"; main :: () { go(); }",
        &[("User", "go :: () -> s64 { return 0; }")],
        Leniency::MainReturnsValue,
    );
    assert!(found.is_empty(), "{found:?}");
    let found = warnings(
        "f :: () -> s64 { return 0; } main :: () { f(); }",
        &[],
        Leniency::MainReturnsValue,
    );
    assert!(found.is_empty(), "{found:?}");
}

#[test]
fn caller_location_outside_a_default_warns() {
    let text = warns_in_the_program_and_not_in_a_module(
        Leniency::CallerLocationOutsideDefault,
        "main :: () { l := #caller_location; }",
        "go :: () { l := #caller_location; }",
    );
    assert!(
        text.contains("`#caller_location` is only allowed as the default value of a parameter"),
        "{text}"
    );
    assert!(text.contains("`#location()`"), "{text}");
    let found = warnings(
        "f :: (loc := #caller_location) {}
        main :: () { f(); l := #location(); }",
        &[],
        Leniency::CallerLocationOutsideDefault,
    );
    assert!(found.is_empty(), "{found:?}");
}

#[test]
fn several_values_in_a_code_argument_warn() {
    let text = warns_in_the_program_and_not_in_a_module(
        Leniency::MultiValueInCodeArgument,
        "f :: (c: Code) #expand { #insert c; }
        main :: () { f(#code { a, b := 1, 2; }); }",
        "f :: (c: Code) #expand { #insert c; }
        go :: () { f(#code { a, b := 1, 2; }); }",
    );
    assert!(
        text.contains("a comma ends a `#code` block that is a call argument"),
        "{text}"
    );
    assert!(text.contains("its own statement"), "{text}");
    let found = warnings(
        "f :: (c: Code) #expand { #insert c; }
        main :: () { f(#code { a := 1; b := 2; }); a, b := 1, 2; }",
        &[],
        Leniency::MultiValueInCodeArgument,
    );
    assert!(found.is_empty(), "{found:?}");
}

#[test]
fn a_spread_next_to_other_varargs_warns() {
    let text = warns_in_the_program_and_not_in_a_module(
        Leniency::SpreadWithOtherArguments,
        "f :: (xs: ..s64) {}
        main :: () { a := s64.[1, 2]; f(0, ..a); }",
        "f :: (xs: ..s64) {}
        go :: () { a := s64.[1, 2]; f(0, ..a); }",
    );
    assert!(
        text.contains("`..` spreads into a variadic parameter by itself"),
        "{text}"
    );
    assert!(
        text.contains("collect all the values into one array"),
        "{text}"
    );
    let found = warnings(
        "f :: (xs: ..s64) {}
        main :: () { a := s64.[1, 2]; f(..a); f(0, 1); }",
        &[],
        Leniency::SpreadWithOtherArguments,
    );
    assert!(found.is_empty(), "{found:?}");
}

#[test]
fn exists_on_an_expression_warns() {
    let text = warns_in_the_program_and_not_in_a_module(
        Leniency::ExistsOnExpression,
        "W :: struct (N: s64) { c :: 1; }
        main :: () { #assert #exists(W(3).c); }",
        "W :: struct (N: s64) { c :: 1; }
        go :: () { #assert #exists(W(3).c); }",
    );
    assert!(
        text.contains("`#exists` takes a name or names joined by `.`"),
        "{text}"
    );
    assert!(text.contains("there is no portable spelling"), "{text}");
    let found = warnings(
        "W :: struct { c :: 1; }
        main :: () { #assert #exists(W.c); }",
        &[],
        Leniency::ExistsOnExpression,
    );
    assert!(found.is_empty(), "{found:?}");
}

#[test]
fn a_procedure_cast_to_an_integer_warns() {
    let text = warns_in_the_program_and_not_in_a_module(
        Leniency::ProcedureCastToInteger,
        "p :: () {}
        main :: () { n := cast(u64) p; }",
        "p :: () {}
        go :: () { n := cast(u64) p; }",
    );
    assert!(
        text.contains("a procedure cannot be cast to `u64` directly"),
        "{text}"
    );
    assert!(text.contains("cast(u64) cast(*void) procedure"), "{text}");
    let found = warnings(
        "p :: () {}
        main :: () { n := cast(u64) cast(*void) p; }",
        &[],
        Leniency::ProcedureCastToInteger,
    );
    assert!(found.is_empty(), "{found:?}");
}

#[test]
fn the_address_of_a_macro_constant_warns() {
    let text = warns_in_the_program_and_not_in_a_module(
        Leniency::AddressOfMacroConstant,
        "m :: (x: u32) -> u8 #expand { p := cast(*u8) *x; return p.*; }
        main :: () { m(5); }",
        "m :: (x: u32) -> u8 #expand { p := cast(*u8) *x; return p.*; }
        go :: () { m(5); }",
    );
    assert!(
        text.contains("`x` is a constant here, and a constant has no address"),
        "{text}"
    );
    assert!(text.contains("copy := x;"), "{text}");
    let found = warnings(
        "m :: (x: u32) -> u8 #expand { p := cast(*u8) *x; return p.*; }
        main :: () { v: u32 = 5; m(v); }",
        &[],
        Leniency::AddressOfMacroConstant,
    );
    assert!(found.is_empty(), "{found:?}");
}

#[test]
fn a_positional_literal_that_leaves_defaults_warns() {
    let text = warns_in_the_program_and_not_in_a_module(
        Leniency::PositionalLiteralDefaults,
        "L :: struct { a: s32; b: s32; c: u32 = 7; }
        main :: () { x: L = .{1, 2}; }",
        "L :: struct { a: s32; b: s32; c: u32 = 7; }
        go :: () { x: L = .{1, 2}; }",
    );
    assert!(
        text.contains("not enough values in this struct literal"),
        "{text}"
    );
    assert!(text.contains("name the members you set"), "{text}");
    for source in [
        "L :: struct { a: s32; b: s32; c: u32 = 7; }
        main :: () { x: L = .{1, 2, 3}; y: L = .{a = 1, b = 2}; }",
        "L :: struct { a: s32 = 1; b: s32 = 2; }
        main :: () { x: L = .{}; }",
    ] {
        let found = warnings(source, &[], Leniency::PositionalLiteralDefaults);
        assert!(found.is_empty(), "{found:?}");
    }
}

#[test]
fn a_positional_literal_missing_a_member_without_a_default_is_an_error() {
    let (result, _) = compile(
        "L :: struct { a: s32; b: s32; c: u32 = 7; }
        main :: () { x: L = .{1}; }",
        &[],
    );
    let error = result.expect_err("a member without a default needs a value");
    assert_eq!(error.message, "not enough values in this struct literal");
}
