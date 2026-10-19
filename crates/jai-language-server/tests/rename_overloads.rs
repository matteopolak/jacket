//! Renaming one overload renames the name of the whole set: every declaration and every call.
mod common;
use common::{at_nth, session, uri};

const OVERLOADS: &str = r#"G :: struct { n: int; }
pick :: (it: $T) -> string { return "poly"; }
pick :: (g: G) -> string { return "G"; }
pick :: (x: s64) -> string { return "s64"; }

main :: () {
    g: G;
    a := pick(g);
    i: s64 = 3;
    b := pick(i);
    c := pick(2.5);
}
"#;

#[test]
fn renaming_one_overload_renames_the_whole_set() {
    let mut s = session();
    s.open(uri(), 1, OVERLOADS.into()).unwrap();
    let at = at_nth(OVERLOADS, "pick ::", 1, 0);
    let edits = s.rename(&uri(), at, "CAP").unwrap().expect("rename");
    let lines: Vec<u32> = edits[0].1.iter().map(|e| e.range.start.line).collect();
    assert_eq!(lines, [1, 2, 3, 7, 9, 10]);
}
