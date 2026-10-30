//! `null`, `true` and `false` always read as literals, so declaring a variable, constant or
//! parameter of that name is an error. A struct member may use them: it is reached as `x.null`.
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
fn members_and_assignments_may_use_them() {
    let source = "S :: struct { null: bool; true: int; }\n\
                  f :: () -> int, bool { return 1, true; }\n\
                  main :: () { s: S; s.null = true; a: int; a, s.null = f(); }\n";
    compile(source).unwrap();
}
