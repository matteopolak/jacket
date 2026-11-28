//! `null`, `true` and `false` always read as literals, so declaring a variable, constant or
//! parameter of that name is an error. A struct member may use them: it is reached as `x.null`.
//! The words that start a statement (`case`, `if`, `for`, `return`, ...) can't be declared
//! either: a statement can't start with them as a name. Nor can `ifx`, `then` and `else`, which
//! are read as part of an `if` or `ifx`. A backticked declaration may use them.
use jaic::diagnostics::Diagnostic;
use jaic::sema::{Compiler, Options, VirtualFs};
use std::{path::Path, rc::Rc};

fn compile(source: &str) -> Result<(), Box<Diagnostic>> {
    let mut fs = VirtualFs::default();

    fs.insert("/main.jai", source.as_bytes());
    let mut options = Options::host();

    options.runtime_support = false;
    let mut compiler = Compiler::new(options, Rc::new(fs));
    compiler.compile_program(Path::new("/main.jai"))
}

#[test]
fn literal_keywords_cannot_be_declared() {
    for (source, word) in [
        (
            "f :: () -> int, *int { return 1, null; }\nmain :: () { a, null := f(); }\n",
            "null",
        ),
        ("main :: () { true := 1; }\n", "true"),
        ("false :: 0;\nmain :: () {}\n", "false"),
        ("main :: () { x: int; x, null := 1, 2; }\n", "null"),
        ("f :: (null: int) {}\nmain :: () { f(1); }\n", "null"),
        (
            "S :: struct { f :: (true: bool) {} }\nmain :: () {}\n",
            "true",
        ),
        (
            "S :: struct { f :: () { null := 1; } }\nmain :: () {}\n",
            "null",
        ),
    ] {
        let error = compile(source).expect_err(source);

        assert_eq!(
            error.message,
            format!("`{word}` is a keyword and cannot be declared"),
            "{source}"
        );
    }
}

#[test]
fn statement_keywords_cannot_be_declared() {
    for (source, word) in [
        ("f :: (case: int) {}\nmain :: () { f(1); }\n", "case"),
        ("main :: () { x, case := 1, 2; }\n", "case"),
        (
            "f :: (a: int, for: int) {}\nmain :: () { f(1, 2); }\n",
            "for",
        ),
        ("main :: () { x, return := 1, 2; }\n", "return"),
        ("f :: (defer: int) {}\nmain :: () { f(1); }\n", "defer"),
        ("f :: (using: int) {}\nmain :: () { f(1); }\n", "using"),
    ] {
        let error = compile(source).expect_err(source);

        assert_eq!(
            error.message,
            format!("`{word}` is a keyword and cannot be declared"),
            "{source}"
        );
    }

    // A backticked declaration is not read as a statement, so it may use the word.
    compile("S :: struct { `remove: int; }\nmain :: () { s: S; s.remove = 1; }\n").unwrap();
}

/// `ifx` always starts an expression, and `then` and `else` are read as part of an `if` that
/// precedes them, so none of them can name a variable, parameter or member.
#[test]
fn if_keywords_cannot_be_declared() {
    for (source, word) in [
        ("main :: () { ifx := 1; }\n", "ifx"),
        ("ifx :: 1;\nmain :: () {}\n", "ifx"),
        ("main :: () { then := 1; if then then = 2; }\n", "then"),
        ("f :: (then: int) {}\nmain :: () { f(1); }\n", "then"),
        ("S :: struct { else: int; }\nmain :: () {}\n", "else"),
    ] {
        let error = compile(source).expect_err(source);

        assert_eq!(
            error.message,
            format!("`{word}` is a keyword and cannot be declared"),
            "{source}"
        );
    }

    compile("S :: struct { `ifx: int; }\nmain :: () { s: S; s.ifx = 1; }\n").unwrap();
}

#[test]
fn members_and_assignments_may_use_them() {
    let source = "S :: struct { null: bool; true: int; }\n\
                  f :: () -> int, bool { return 1, true; }\n\
                  main :: () { s: S; s.null = true; a: int; a, s.null = f(); }\n";

    compile(source).unwrap();
}
