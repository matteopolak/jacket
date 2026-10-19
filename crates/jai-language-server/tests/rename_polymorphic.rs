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

const BAKED: &str = r"Grid :: struct (T: Type, W: int) {
    cells: [W] T;
}

first :: (g: Grid($T, $W)) -> T {
    sum: T;
    return sum;
}

first_f :: #bake_constants first(T = float32);

main :: () {
    small: Grid(float32, 2);
    x := first_f(small);
}
";

#[test]
fn a_type_variable_is_renamed_where_bake_constants_names_it_and_nowhere_else() {
    let mut s = session();
    s.open(uri(), 1, BAKED.into()).unwrap();
    let at = at_nth(BAKED, "$T", 0, 1);
    let edits = s.rename(&uri(), at, "E").unwrap().expect("rename");
    let mut lines: Vec<u32> = edits[0].1.iter().map(|e| e.range.start.line).collect();
    lines.sort_unstable();
    // `$T`, `-> T`, `sum: T` and `T = float32`, not the struct's own `T`.
    assert_eq!(lines, [4, 4, 5, 9], "{edits:?}");
}

const BARE: &str = r"Base :: struct ($Reading: bool) {
    depth: int;
}

depth_of :: (r: Base) -> int {
    return r.depth;
}

main :: () {
    w: Base(true);
    x := depth_of(w);
}
";

#[test]
fn a_parameter_typed_with_a_bare_polymorphic_struct_is_renamed_in_the_body() {
    let mut s = session();
    s.open(uri(), 1, BARE.into()).unwrap();
    let at = at_nth(BARE, "r: Base", 0, 0);
    let edits = s.rename(&uri(), at, "n").unwrap().expect("rename");
    assert_eq!(edits[0].1.len(), 2, "{edits:?}");
}
