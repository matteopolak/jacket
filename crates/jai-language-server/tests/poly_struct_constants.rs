//! A constant declared in a polymorphic struct (`count :: N;`) is reached through an instance
//! like any field: definition, hover and references land on the declaration in the struct.
use test_support::lsp::{at_nth, open, uri};

const PROGRAM: &str = "\
Buf :: struct($N: int) {
    count :: N;
    data: [N] u8;
}

main :: () {
    b: Buf(4);
    x := b.count;
    y := b.data;
}
";

fn lines(found: &[jai_language_server::Location]) -> Vec<u32> {
    let mut lines: Vec<u32> = found.iter().map(|l| l.range.start.line).collect();
    lines.sort_unstable();
    lines.dedup();
    lines
}

#[test]
fn go_to_definition_of_a_constant_reaches_the_declaration() {
    let s = open(PROGRAM);
    let found = s
        .definition(&uri(), at_nth(PROGRAM, "b.count", 0, 2))
        .unwrap();
    assert_eq!(lines(&found), [1], "{found:?}");
}

#[test]
fn go_to_definition_of_a_field_still_works() {
    let s = open(PROGRAM);
    let found = s
        .definition(&uri(), at_nth(PROGRAM, "b.data", 0, 2))
        .unwrap();
    assert_eq!(lines(&found), [2], "{found:?}");
}

#[test]
fn hover_on_a_constant_shows_it() {
    let s = open(PROGRAM);
    let hover = s
        .hover(&uri(), at_nth(PROGRAM, "b.count", 0, 2))
        .unwrap()
        .expect("hover")
        .contents
        .value;
    assert!(hover.contains("count"), "{hover}");
}

#[test]
fn completion_offers_the_constants_of_a_using_base() {
    const SOURCE: &str = "\
Plane :: struct(data_type: Type) {}

Grid :: struct(T: Type) {
    using #as _base :: Plane(T).{};
    cells: [4] T;
}

get :: (a: *Grid, v: a.data_type) {}

main :: () {}
";
    let s = open(SOURCE);
    let list = s
        .completion(&uri(), at_nth(SOURCE, "a.data_type", 0, 2))
        .unwrap();
    assert!(
        list.items.iter().any(|i| i.label == "data_type"),
        "{:?}",
        list.items.iter().map(|i| &i.label).collect::<Vec<_>>()
    );
}

#[test]
fn references_of_a_constant_include_the_declaration_and_the_use() {
    let s = open(PROGRAM);
    let refs = s
        .references(&uri(), at_nth(PROGRAM, "b.count", 0, 2), true)
        .unwrap();
    assert_eq!(lines(&refs), [1, 7], "{refs:?}");
}
