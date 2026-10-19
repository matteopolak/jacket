//! Renaming a parameter of a polymorphic procedure reaches the uses in its body.
mod common;
use common::{at_nth, session, uri};

const PROGRAM: &str = r"twice :: (x: $T) -> type_of(x) {
    return x + x;
}

main :: () {
    y := twice(21);
}
";

#[test]
fn a_polymorphic_parameter_is_renamed_where_the_body_uses_it() {
    let mut s = session();
    s.open(uri(), 1, PROGRAM.into()).unwrap();
    let at = at_nth(PROGRAM, "x: $T", 0, 0);
    let edits = s.rename(&uri(), at, "n").unwrap().expect("rename");
    assert_eq!(edits[0].1.len(), 4, "{edits:?}");
}
