//! A parameter of a polymorphic struct is renamed where the body uses it.
mod common;
use common::{at_nth, session, uri};

const PROGRAM: &str = r"Grid :: struct (T: Type, W: int, H: int) {
    cells: [H][W] T;
}

main :: () {
    g: Grid(float32, 2, 3);
}
";

#[test]
fn a_struct_parameter_is_renamed_in_the_body() {
    let mut s = session();
    s.open(uri(), 1, PROGRAM.into()).unwrap();
    for (name, needle) in [("H", "H: int"), ("W", "W: int"), ("T", "T: Type")] {
        let at = at_nth(PROGRAM, needle, 0, 0);
        let edits = s.rename(&uri(), at, "N").unwrap().expect("rename");
        assert_eq!(edits[0].1.len(), 2, "{name}: {edits:?}");
    }
}
