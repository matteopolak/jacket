//! An operator overload is also found in the module that declares an operand's struct, so a
//! namespaced import (`V :: #import "Vec"`) is enough to use `a + b`. Using it that way warns
//! with the portable spelling (`V.operator +(a, b)`); an operator nothing reaches names the
//! import or alias that has it.
use jaic::sema::{Compiler, Options, VirtualFs};
use jaic::source::{Applicability, Diagnostic, Edit};
use std::{
    path::{Path, PathBuf},
    rc::Rc,
};

const VEC: &str = "
Vec2 :: struct { x: int; y: int; }
operator + :: (a: Vec2, b: Vec2) -> Vec2 { return .{a.x + b.x, a.y + b.y}; }
operator - :: (a: Vec2) -> Vec2 { return .{-a.x, -a.y}; }
operator == :: (a: Vec2, b: Vec2) -> bool { return a.x == b.x && a.y == b.y; }
operator [] :: (a: Vec2, i: int) -> int { return ifx i == 0 then a.x else a.y; }

Scale :: struct { k: int; }
operator * :: (a: Vec2, s: int) -> Vec2 { return .{a.x * s, a.y * s}; }
operator * :: (a: Vec2, s: Scale) -> Vec2 { return .{a.x * s.k, a.y * s.k}; }

Grid :: struct { cells: [4] int; }
operator *[] :: (g: *Grid, i: int) -> *int { return *g.cells[i]; }
";

/// A `Thing` whose module has no operators, and a module with the operator for it.
const OTHER: &str = "Thing :: struct { n: int; }";

const OPS: &str = "O :: #import \"Other\";
operator + :: (a: O.Thing, b: O.Thing) -> O.Thing { return .{a.n + b.n}; }";

fn compile(main: &str) -> (Result<(), Box<Diagnostic>>, Vec<Diagnostic>, Compiler) {
    let mut fs = VirtualFs::default();
    fs.insert("/modules/Vec/module.jai", VEC.as_bytes());
    fs.insert("/modules/Other/module.jai", OTHER.as_bytes());
    fs.insert("/modules/Ops/module.jai", OPS.as_bytes());
    fs.insert("/main.jai", main.as_bytes());
    let mut options = Options::host();
    options.runtime_support = false;
    options.import_paths = vec![PathBuf::from("/modules")];
    // The modules above are also the standard library that imports are suggested from.
    options.preload_stdlib = Some(PathBuf::from("/modules"));
    let mut compiler = Compiler::new(options, Rc::new(fs));
    let result = compiler.compile_program(Path::new("/main.jai"));
    let warnings = compiler.diag.warnings.clone();
    (result, warnings, compiler)
}

fn warnings_of(main: &str) -> Vec<Diagnostic> {
    let (result, warnings, compiler) = compile(main);
    if let Err(error) = result {
        panic!("{}", error.render(&compiler.src.sources));
    }
    warnings
        .into_iter()
        .filter(|w| w.message.contains("is not in scope"))
        .collect()
}

fn replacement(d: &Diagnostic) -> (&str, Applicability, bool) {
    let s = &d.suggestions()[0];
    let text = s.replacements().next().expect("a replacement").1;
    let imports = s.edits.iter().any(|e| matches!(e, Edit::Import(_)));
    (text, s.applicability, imports)
}

#[test]
fn a_namespaced_import_finds_the_operator_of_the_operand_type() {
    let warnings = warnings_of(
        "V :: #import \"Vec\";
        main :: () {
            a, b: V.Vec2;
            c := a + b;
            c += b;
            d := -a;
            e := a[1];
            f := a != b;
        }",
    );
    let messages: Vec<&str> = warnings.iter().map(|w| w.message.as_str()).collect();
    assert_eq!(
        messages,
        [
            "`operator +` is not in scope here: using the one declared with `Vec2`",
            "`operator +` is not in scope here: using the one declared with `Vec2`",
            "`operator -` is not in scope here: using the one declared with `Vec2`",
            "`operator []` is not in scope here: using the one declared with `Vec2`",
            "`operator ==` is not in scope here: using the one declared with `Vec2`",
        ]
    );
}

#[test]
fn the_warning_suggests_the_portable_spelling() {
    let warnings = warnings_of(
        "V :: #import \"Vec\";
        main :: () {
            a, b: V.Vec2;
            c := a + b;
            c += b;
            d := -a;
            f := a != b;
        }",
    );
    assert_eq!(
        replacement(&warnings[0]),
        (
            "V.operator +(a, b)",
            Applicability::MachineApplicable,
            false
        )
    );
    // The statement `c += b` becomes an assignment: a guess, the author checks it.
    assert_eq!(
        replacement(&warnings[1]),
        (
            "c = V.operator +(c, b)",
            Applicability::MaybeIncorrect,
            false
        )
    );
    assert_eq!(
        replacement(&warnings[2]),
        ("V.operator -(a)", Applicability::MachineApplicable, false)
    );
    assert_eq!(
        replacement(&warnings[3]),
        ("!V.operator ==(a, b)", Applicability::MaybeIncorrect, false)
    );
    assert!(
        warnings[0]
            .help
            .iter()
            .any(|h| h == "or add `using V;` to this scope")
    );
    assert_eq!(
        warnings[0].kind.code(),
        Some("operator-out-of-scope"),
        "tools can tell it from other warnings"
    );
}

#[test]
fn imports_that_bring_the_operator_into_scope_do_not_warn() {
    for header in [
        "#import \"Vec\"; V :: Vec2;",
        "V :: #import \"Vec\"; using V;",
        "using V :: #import \"Vec\";",
    ] {
        let warnings = warnings_of(&format!(
            "{header}
            main :: () {{ a, b: Vec2; c := a + b; d := -a; e := a[0]; f := a == b; }}"
        ));
        assert!(warnings.is_empty(), "{header}: {warnings:?}");
    }
}

#[test]
fn an_overload_in_scope_wins_over_the_type_module() {
    let warnings = warnings_of(
        "V :: #import \"Vec\";
        operator + :: (a: V.Vec2, b: V.Vec2) -> V.Vec2 { return a; }
        main :: () { a, b: V.Vec2; c := a + b; }",
    );
    assert!(warnings.is_empty(), "{warnings:?}");
}

#[test]
fn a_namespaced_alias_that_has_the_operator_is_suggested_when_the_type_module_lacks_it() {
    let (result, _, compiler) = compile(
        "T :: #import \"Other\"; P :: #import \"Ops\";
        main :: () { a, b: T.Thing; c := a + b; }",
    );
    let error = result.expect_err("the operator is in Ops, not with the type");
    let text = error.render(&compiler.src.sources);
    assert!(
        text.contains("`P` declares `operator +`: call `P.operator +(..)` or add `using P;`"),
        "{text}"
    );
    assert_eq!(
        replacement(&error),
        ("P.operator +(a, b)", Applicability::MaybeIncorrect, false)
    );
}

#[test]
fn an_operator_no_import_has_still_fails_without_a_guess() {
    let (result, _, compiler) = compile(
        "T :: #import \"Other\";
        main :: () { a, b: T.Thing; c := a + b; }",
    );
    let error = result.expect_err("no overload anywhere in reach");
    let text = error.render(&compiler.src.sources);
    assert!(
        text.contains("no `operator +` for `Thing` and `Thing`"),
        "{text}"
    );
    assert!(!text.contains("declares `operator +`"), "{text}");
}

#[test]
fn a_unary_operator_nothing_reaches_still_fails() {
    let (result, _, compiler) = compile(
        "T :: #import \"Other\";
        main :: () { a: T.Thing; c := -a; }",
    );
    let text = result
        .expect_err("no overload")
        .render(&compiler.src.sources);
    assert!(text.contains("cannot negate Thing"), "{text}");
}

#[test]
fn a_mismatch_names_the_operands_and_lists_the_overloads_as_notes() {
    let (result, _, compiler) = compile(
        "V :: #import \"Vec\";
        using V;
        T :: #import \"Other\";
        main :: () { a: V.Vec2; b: T.Thing; c := a + b; }",
    );
    let text = result
        .expect_err("no overload takes both")
        .render(&compiler.src.sources);
    assert!(
        text.contains("no `operator +` for `Vec2` and `Thing`"),
        "{text}"
    );
    assert!(!text.contains("in call to"), "{text}");
    assert!(text.contains("`operator +` is declared here"), "{text}");
}

fn import_suggestions(d: &Diagnostic) -> Vec<String> {
    d.suggestions()
        .iter()
        .flat_map(jaic::source::Suggestion::imports)
        .map(|i| i.module.clone())
        .collect()
}

#[test]
fn an_unrelated_struct_gets_no_import_suggestion() {
    // `Vec` and `Ops` declare `operator +`, for other types.
    let (result, _, compiler) = compile(
        "Mine :: struct { n: int; }
        main :: () { a, b: Mine; c := a + b; }",
    );
    let error = result.expect_err("no overload");
    assert!(import_suggestions(&error).is_empty(), "{error:?}");
    let text = error.render(&compiler.src.sources);
    assert!(!text.contains("is declared in the"), "{text}");
}

#[test]
fn a_module_with_an_overload_for_the_operands_is_suggested() {
    let (result, _, compiler) = compile(
        "T :: #import \"Other\";
        main :: () { a, b: T.Thing; c := a + b; }",
    );
    let error = result.expect_err("Ops is not imported");
    // `Ops` takes `Thing`; `Vec` declares `operator +` for another type.
    assert_eq!(import_suggestions(&error), ["Ops"]);
    let text = error.render(&compiler.src.sources);
    assert!(
        text.contains("`operator +` is declared in the `Ops` module"),
        "{text}"
    );
}

#[test]
fn a_unary_operator_no_module_overloads_gets_no_import_suggestion() {
    let (result, _, _) = compile(
        "T :: #import \"Other\";
        main :: () { a: T.Thing; c := -a; }",
    );
    let error = result.expect_err("no `operator -` for Thing");
    assert!(import_suggestions(&error).is_empty(), "{error:?}");
}

#[test]
fn a_literal_operand_takes_its_type_from_the_overload_of_the_type_module() {
    let warnings = warnings_of(
        "V :: #import \"Vec\";
        main :: () {
            a: V.Vec2;
            b := a * 2;
            c := a * .{3};
            d := a * V.Scale.{4};
        }",
    );
    // `.{3}` is a `Scale` (as in scope), not the `Vec2` of the other operand.
    assert_eq!(warnings.len(), 3, "{warnings:?}");
    assert_eq!(
        replacement(&warnings[0]),
        (
            "V.operator *(a, 2)",
            Applicability::MachineApplicable,
            false
        )
    );
    assert_eq!(
        replacement(&warnings[1]),
        (
            "V.operator *(a, .{3})",
            Applicability::MachineApplicable,
            false
        )
    );
}

#[test]
fn the_pointer_index_operator_is_found_through_the_type_module() {
    let warnings = warnings_of(
        "V :: #import \"Vec\";
        main :: () {
            g: V.Grid;
            g[1] = 5;
            x := g[1];
            p := *g[2];
            q := *g;
            q[3] += 1;
        }",
    );
    assert_eq!(warnings.len(), 4, "{warnings:?}");
    for w in &warnings {
        assert_eq!(
            w.message,
            "`operator *[]` is not in scope here: using the one declared with `Grid`"
        );
        // `V.operator *[](..)` does not parse, so only `using` is offered.
        assert_eq!(w.help, ["or add `using V;` to this scope"]);
    }
}

#[test]
fn the_pointer_index_operator_in_scope_is_not_a_warning() {
    let warnings = warnings_of(
        "V :: #import \"Vec\"; using V;
        main :: () { g: Grid; g[1] = 5; x := g[1]; }",
    );
    assert!(warnings.is_empty(), "{warnings:?}");
}
