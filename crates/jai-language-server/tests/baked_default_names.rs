//! A call that bakes a defaulted `$` parameter does not declare the callee's `$T` at the call:
//! a type argument whose name starts with `T` still hovers and goes to its own declaration.
mod common;
use common::{at, at_offset, open, uri};

const PROGRAM: &str = "make :: ($T: Type, $n := 1) -> int { return n; }
Table :: struct (K: Type) { k: K; }

main :: () {
    x := make(Table(int));
}
";

#[test]
fn a_type_argument_is_not_taken_for_the_callees_parameter() {
    let s = open(PROGRAM);
    let use_site = at_offset(PROGRAM, "Table(int)", 1);
    let defs = s.definition(&uri(), use_site).unwrap();
    assert_eq!(defs.len(), 1, "{defs:?}");
    assert_eq!(defs[0].range.start, at(PROGRAM, "Table ::"), "{defs:?}");
    let hover = s.hover(&uri(), use_site).unwrap().expect("hover");
    assert!(
        hover.contents.value.starts_with("Table(s64) ::"),
        "{}",
        hover.contents.value
    );
}
