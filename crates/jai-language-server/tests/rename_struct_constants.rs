//! A constant declared in a struct is found where it is named through the type, a field's type or
//! an instance, so references and renames reach every use.
mod common;
use common::{at_nth, session, uri};

const PROGRAM: &str = r"Map :: struct {
    LIMIT :: 70;
}

Box :: struct {
    m: Map;
}

main :: () {
    a := Map.LIMIT;
    b := Box.m.LIMIT;
    m: Map;
    c := m.LIMIT;
}
";

#[test]
fn a_struct_constant_is_renamed_in_every_use() {
    let mut s = session();
    s.open(uri(), 1, PROGRAM.into()).unwrap();
    let at = at_nth(PROGRAM, "LIMIT ::", 0, 0);
    let edits = s.rename(&uri(), at, "CAP").unwrap().expect("rename");
    let lines: Vec<u32> = edits[0].1.iter().map(|e| e.range.start.line).collect();
    assert_eq!(lines, [1, 9, 10, 12]);
}

const ANONYMOUS: &str = r"using _ :: struct {
    LIMIT :: 70;
}

main :: () {
    a := LIMIT;
    b := LIMIT + 1;
}
";

#[test]
fn a_constant_of_an_anonymous_using_struct_is_renamed_in_every_use() {
    let mut s = session();
    s.open(uri(), 1, ANONYMOUS.into()).unwrap();
    let at = at_nth(ANONYMOUS, "LIMIT ::", 0, 0);
    let edits = s.rename(&uri(), at, "CAP").unwrap().expect("rename");
    let lines: Vec<u32> = edits[0].1.iter().map(|e| e.range.start.line).collect();
    assert_eq!(lines, [1, 5, 6]);
}

#[test]
fn a_constant_of_an_anonymous_using_struct_has_hover_on_its_uses() {
    let mut s = session();
    s.open(uri(), 1, ANONYMOUS.into()).unwrap();
    let at = at_nth(ANONYMOUS, "LIMIT;", 0, 0);
    let hover = s.hover(&uri(), at).unwrap().expect("hover");
    assert!(hover.contents.value.contains("LIMIT"), "{hover:?}");
}
