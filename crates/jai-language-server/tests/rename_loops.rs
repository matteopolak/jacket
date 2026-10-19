//! A loop's implicit `it` has no declaration to rewrite, and a named loop variable is declared
//! where it is written, not at the first matching letters of the loop.
mod common;
use common::{at_nth, session, uri};

const PROGRAM: &str = r"main :: () {
    xs: [3] int;
    for xs print(it);
    for o: xs print(o);
}

print :: (n: int) {}
";

#[test]
fn only_a_written_loop_variable_can_be_renamed() {
    let mut s = session();
    s.open(uri(), 1, PROGRAM.into()).unwrap();
    let implicit = at_nth(PROGRAM, "(it)", 0, 1);
    assert!(s.prepare_rename(&uri(), implicit).unwrap().is_none());
    let written = at_nth(PROGRAM, "o: xs", 0, 0);
    let edits = s.rename(&uri(), written, "n").unwrap().expect("rename");
    let starts: Vec<_> = edits[0].1.iter().map(|e| e.range.start).collect();
    assert_eq!(starts, [written, at_nth(PROGRAM, "(o)", 0, 1)]);
}
