//! A local named only inside `type_of` in a constant context (`#assert type_of(b) == float64;`) is a use of it, so a rename reaches it.
mod common;
use common::{at_nth, session, uri};

const PROGRAM: &str = r"main :: () {
    b := 2.5;
    #assert type_of(b) == float64;
    #assert type_of(b) != float32;
}
";

#[test]
fn a_local_is_renamed_in_static_assertions() {
    let mut s = session();
    s.open(uri(), 1, PROGRAM.into()).unwrap();
    let at = at_nth(PROGRAM, "b :=", 0, 0);
    let edits = s.rename(&uri(), at, "n").unwrap().expect("rename");
    assert_eq!(edits[0].1.len(), 3, "{edits:?}");
}
