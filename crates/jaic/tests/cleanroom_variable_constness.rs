//! A variable is never a constant: only `::` and `: T :` declarations name constants.
use jaic::sema::{Compiler, Options, ProgramSource, VirtualFs};
use std::rc::Rc;

fn compile(source: &str) -> Result<(), String> {
    let mut options = Options::host();
    options.runtime_support = false;
    let mut c = Compiler::new(options, Rc::new(VirtualFs::default()));
    c.compile_sources(&[ProgramSource::String(source.into())])
        .map_err(|error| error.render(&c.sources))
}

fn accepts(source: &str) {
    if let Err(message) = compile(source) {
        panic!("{message}");
    }
}

fn rejects(source: &str, expected: &str) {
    match compile(source) {
        Ok(()) => panic!("accepted, expected `{expected}`: {source}"),
        Err(message) => assert!(
            message.contains(expected),
            "expected `{expected}` in:\n{message}"
        ),
    }
}

#[test]
fn if_condition_cannot_read_a_variable() {
    rejects(
        "G := 5; #if G == 5 { X :: 1; }",
        "#if condition must be a constant, but `G` is a variable",
    );
    rejects(
        "g : int = 5; #if g == 5 { X :: 1; }",
        "#if condition must be a constant, but `g` is a variable",
    );
}

#[test]
fn assert_condition_may_read_what_compile_time_code_wrote() {
    accepts("G := 5; #assert G == 5;");
}

#[test]
fn constant_declaration_cannot_read_a_variable() {
    rejects(
        "G := 5; C :: G;",
        "constant declaration needs a constant value, but `G` is a variable",
    );
    rejects(
        "G := 5; C : int : G + 1;",
        "constant declaration needs a constant value, but `G` is a variable",
    );
    rejects(
        "G := 5; main :: () { c :: G; x := c; }",
        "constant declaration needs a constant value, but `G` is a variable",
    );
}

#[test]
fn other_constant_contexts_cannot_read_a_variable() {
    rejects("G := 5; E :: enum { A :: G; }", "but `G` is a variable");
    rejects("G := 5; main :: () { arr: [G] int; }", "constant");
    rejects(
        "G := 5; f :: ($N: int) {} main :: () { f(G); }",
        "compile-time constant",
    );
    rejects("G := \"X :: 1;\"; #insert G;", "but `G` is a variable");
}

#[test]
fn run_may_read_and_write_variables() {
    accepts("G := 5; X :: #run G + 1; #assert X == 6;");
    accepts("G := 5; #run { G = 7; }");
}

#[test]
fn the_address_of_a_variable_is_constant() {
    accepts("G := 5; P :: *G; S :: struct { a: int; b: int; } H: S; Q :: *H.b;");
    rejects("G := 5; A: [2]int; P :: *A[G];", "is a variable");
}

#[test]
fn type_queries_do_not_read_a_variable() {
    accepts("G := 5; T :: type_of(G); S :: size_of(type_of(G)); #assert S == 8;");
}

#[test]
fn constant_declarations_stay_constant() {
    accepts("K :: 5; arr: [K] int; #assert K == 5;");
    accepts("K : int : 5; arr: [K] int;");
    accepts("S :: struct { K :: 3; } arr: [S.K] int;");
    rejects("K :: 5; main :: () { K = 6; }", "cannot assign to constant");
    rejects(
        "K : int : 5; main :: () { K = 6; }",
        "cannot assign to constant",
    );
}

#[test]
fn variables_stay_mutable() {
    accepts("S :: struct { x := 5; } main :: () { s: S; s.x = 9; }");
    accepts("main :: () { x := 5; x = 6; }");
    accepts("G := 5; main :: () { G = 6; }");
    accepts("g := 5; main :: () { g = 6; }");
}
