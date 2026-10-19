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
