//! Each construct Jacket accepts although the official compiler rejects it is a warning with a
//! code of its own and the shared note (`docs/language/leniencies.md`); the same construct in an
//! imported module is not reported.
use jaic::leniency::{Leniency, NOTE};
use jaic::sema::{Compiler, Options, VirtualFs};
use jaic::source::Diagnostic;
use std::{path::Path, rc::Rc};

/// A compile of `main` (`/main.jai`) with `modules` (name, source) importable.
fn compile(main: &str, modules: &[(&str, &str)]) -> (Result<(), Box<Diagnostic>>, Compiler) {
    let mut fs = VirtualFs::default();
    for (name, source) in modules {
        fs.insert(&format!("/modules/{name}/module.jai"), source.as_bytes());
    }
    fs.insert("/main.jai", main.as_bytes());
    let mut options = Options::host();
    options.runtime_support = false;
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
