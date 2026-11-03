//! Go to declaration stops at the name's own declaration (an alias), go to implementation lists
//! the procedures with a body that share the name.
use jai_language_server::{Location, Position, Session};
use test_support::lsp::{file_uri, lean_session};

const SOURCE: &str = "\
Alias :: helper;
helper :: (a: int) -> int { return a; }
helper :: (a: float) -> float { return a; }
ext :: (x: int) -> int #foreign lib \"ext\";
Count :: struct { n: int; }
main :: () {
    Alias(1);
    helper(2);
    ext(3);
}
";

fn open() -> (Session, jai_language_server::DocumentUri) {
    let mut session = lean_session();
    let uri = file_uri("/nav/main.jai");
    session.open(uri.clone(), 1, SOURCE.into()).unwrap();
    (session, uri)
}

fn lines(found: &[Location]) -> Vec<u32> {
    found.iter().map(|l| l.range.start.line).collect()
}

fn at(line: u32, character: u32) -> Position {
    Position {
        line,
        character,
    }
}

#[test]
fn declaration_is_the_alias_where_definition_is_its_target() {
    let (session, uri) = open();
    assert_eq!(lines(&session.declaration(&uri, at(6, 6)).unwrap()), [0]);
    assert_eq!(lines(&session.definition(&uri, at(6, 6)).unwrap()), [1]);
}

#[test]
fn declaration_of_a_plain_name_is_its_declaration() {
    let (session, uri) = open();
    assert_eq!(lines(&session.declaration(&uri, at(8, 5)).unwrap()), [3]);
}

#[test]
fn implementation_lists_every_overload_with_a_body() {
    let (session, uri) = open();
    let from_call = session.implementation(&uri, at(7, 6)).unwrap();
    assert_eq!(lines(&from_call), [1, 2]);
    let from_declaration = session.implementation(&uri, at(2, 2)).unwrap();
    assert_eq!(lines(&from_declaration), [1, 2]);
}

#[test]
fn a_foreign_procedure_and_a_type_have_no_implementation() {
    let (session, uri) = open();
    assert!(session.implementation(&uri, at(8, 5)).unwrap().is_empty());
    assert!(session.implementation(&uri, at(4, 1)).unwrap().is_empty());
}

#[test]
fn neither_compiles_in_a_request() {
    let (session, uri) = open();
    session.defer_compiles(true);
    let compiles = session.compile_counts().0;
    session.declaration(&uri, at(6, 6)).unwrap();
    session.implementation(&uri, at(7, 6)).unwrap();
    assert_eq!(session.compile_counts().0, compiles);
}
