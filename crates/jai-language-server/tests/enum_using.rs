//! `using Other;` in an enum body lends `Other`'s names to the enum: every editor answer for a
//! borrowed name is the answer for the member it was borrowed from, and none lands on the
//! `using` line.
use test_support::lsp::{at_nth, session, uri};

const PROGRAM: &str = "\
Colour :: enum {
    Red;
    Green;
}

Shade :: enum {
    using Colour;
    Dark :: Green;
}

main :: () {
    a := Shade.Red;
    b := Colour.Red;
    c := Shade.Green;
}
";

fn open() -> jai_language_server::Session {
    let mut s = session();
    s.open(uri(), 1, PROGRAM.into()).unwrap();
    s
}

/// The line numbers of the locations.
fn lines(found: &[jai_language_server::Location]) -> Vec<u32> {
    let mut lines: Vec<u32> = found.iter().map(|l| l.range.start.line).collect();
    lines.sort_unstable();
    lines.dedup();
    lines
}

#[test]
fn go_to_definition_of_a_borrowed_name_reaches_the_original_member() {
    let s = open();
    // `Shade.Red` in `main`: the `Red` after the dot.
    let use_at = at_nth(PROGRAM, "Shade.Red", 0, 6);
    let found = s.definition(&uri(), use_at).unwrap();
    assert_eq!(lines(&found), [1], "{found:?}");
}

#[test]
fn hover_on_a_borrowed_name_shows_the_original_member() {
    let s = open();
    let use_at = at_nth(PROGRAM, "Shade.Red", 0, 6);
    let hover = s
        .hover(&uri(), use_at)
        .unwrap()
        .expect("hover")
        .contents
        .value;
    assert!(hover.contains("Red"), "{hover}");
    assert!(!hover.contains("using"), "{hover}");
}

#[test]
fn a_borrowed_name_used_inside_the_enum_body_leads_to_the_original_member() {
    let s = open();
    let inside = at_nth(PROGRAM, "Dark :: Green", 0, 8);
    let found = s.definition(&uri(), inside).unwrap();
    assert_eq!(lines(&found), [2], "{found:?}");
    let hover = s
        .hover(&uri(), inside)
        .unwrap()
        .expect("hover")
        .contents
        .value;
    assert!(hover.contains("Green"), "{hover}");
    let refs = s.references(&uri(), inside, true).unwrap();
    assert_eq!(lines(&refs), [2, 7, 13], "{refs:?}");
    assert!(
        !lines(&refs).contains(&6),
        "the `using` line is not a use: {refs:?}"
    );
}

#[test]
fn references_to_a_member_include_the_uses_through_the_enum_that_borrowed_it() {
    let s = open();
    let declared = at_nth(PROGRAM, "Red;", 0, 0);
    let refs = s.references(&uri(), declared, true).unwrap();
    // The declaration and both uses; the `using` line is not a use of the name.
    assert_eq!(lines(&refs), [1, 11, 12], "{refs:?}");
    // The same from a use through the borrower.
    let use_at = at_nth(PROGRAM, "Shade.Red", 0, 6);
    let from_use = s.references(&uri(), use_at, true).unwrap();
    assert_eq!(lines(&from_use), [1, 11, 12], "{from_use:?}");
    assert!(!lines(&from_use).contains(&6), "{from_use:?}");
}

#[test]
fn renaming_a_member_renames_its_borrowed_uses_and_not_the_using_line() {
    let s = open();
    let declared = at_nth(PROGRAM, "Red;", 0, 0);
    let edits = s
        .rename(&uri(), declared, "Crimson")
        .unwrap()
        .expect("rename");
    let all: Vec<_> = edits.iter().flat_map(|(_, e)| e.iter()).collect();
    let mut edited: Vec<u32> = all.iter().map(|e| e.range.start.line).collect();
    edited.sort_unstable();
    assert_eq!(edited, [1, 11, 12], "{all:?}");
    // Renaming from the borrowed use reaches the same places.
    let use_at = at_nth(PROGRAM, "Shade.Red", 0, 6);
    let edits = s
        .rename(&uri(), use_at, "Crimson")
        .unwrap()
        .expect("rename");
    let total: usize = edits.iter().map(|(_, e)| e.len()).sum();
    assert_eq!(total, 3);
}

#[test]
fn bare_and_qualified_uses_of_a_member_are_uses_of_the_same_member() {
    let prog = "E :: enum {\n    A;\n    B :: A;\n}\nmain :: () { x := E.A; }\n";
    let mut s = session();
    s.open(uri(), 1, prog.into()).unwrap();
    for at in [
        at_nth(prog, "A;", 0, 0),
        at_nth(prog, "B :: A", 0, 5),
        at_nth(prog, "E.A", 0, 2),
    ] {
        let refs = s.references(&uri(), at, true).unwrap();
        assert_eq!(lines(&refs), [1, 2, 4], "{refs:?}");
    }
}

#[test]
fn renaming_from_a_bare_use_in_the_enum_body_reaches_every_use() {
    let s = open();
    let bare = at_nth(PROGRAM, "Dark :: Green", 0, 8);
    let edits = s.rename(&uri(), bare, "Lime").unwrap().expect("rename");
    let mut edited: Vec<u32> = edits
        .iter()
        .flat_map(|(_, e)| e.iter().map(|e| e.range.start.line))
        .collect();
    edited.sort_unstable();
    assert_eq!(edited, [2, 7, 13]);
}
