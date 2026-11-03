//! A local name and its uses are edited together; nothing else is linked.
use jai_language_server::{DocumentUri, Position, Session};
use test_support::lsp::{file_uri, lean_session};

const SOURCE: &str = "\
Thing :: struct { count: int; }
total := 10;
main :: (limit: int) {
    sum := limit + 1;
    other := sum * 2;
    t: Thing;
    t.count = sum;
    print(\"%\", other);
}
";

fn open() -> (Session, DocumentUri) {
    let mut session = lean_session();
    let uri = file_uri("/linked/main.jai");
    session.open(uri.clone(), 1, SOURCE.into()).unwrap();
    (session, uri)
}

fn at(line: u32, character: u32) -> Position {
    Position {
        line,
        character,
    }
}

fn lines(session: &Session, uri: &DocumentUri, position: Position) -> Vec<(u32, u32)> {
    session
        .linked_editing_ranges(uri, position)
        .unwrap()
        .iter()
        .map(|r| (r.start.line, r.start.character))
        .collect()
}

#[test]
fn a_local_and_its_uses_are_linked_from_any_of_them() {
    let (session, uri) = open();
    let expected = vec![(3, 4), (4, 13), (6, 14)];
    assert_eq!(lines(&session, &uri, at(3, 5)), expected);
    assert_eq!(lines(&session, &uri, at(4, 13)), expected);
    assert_eq!(lines(&session, &uri, at(6, 16)), expected);
}

#[test]
fn a_parameter_is_linked_with_its_uses() {
    let (session, uri) = open();
    assert_eq!(lines(&session, &uri, at(2, 9)), [(2, 9), (3, 11)]);
}

#[test]
fn globals_members_and_types_are_not_linked() {
    let (session, uri) = open();
    assert!(lines(&session, &uri, at(1, 2)).is_empty(), "a global");
    assert!(lines(&session, &uri, at(6, 6)).is_empty(), "a member");
    assert!(lines(&session, &uri, at(5, 6)).is_empty(), "a type");
    assert!(lines(&session, &uri, at(0, 1)).is_empty(), "a declaration");
}

#[test]
fn without_a_compile_of_the_text_nothing_is_linked() {
    let (session, uri) = open();
    session.defer_compiles(true);
    let compiles = session.compile_counts().0;
    assert!(lines(&session, &uri, at(3, 5)).is_empty());
    assert_eq!(session.compile_counts().0, compiles, "no compile");
}
