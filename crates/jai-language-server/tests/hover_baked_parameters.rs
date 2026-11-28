//! A procedure's parameter hovers and goes to its own declaration inside the procedure, whether
//! or not a `#bake_arguments` copy of the body (which binds the parameter as a constant) was
//! checked before or after the procedure itself.
use jai_language_server::Session;
use test_support::lsp::{at, at_offset, session, uri};

const BAKE_AFTER: &str = "weighted :: (a: int, b: int) -> int {
    return a * 10 + b;
}
weighted_a :: #bake_arguments weighted(a = 1);
weighted_ab :: #bake_arguments weighted(a = 1, b = 2);

main :: () { x := weighted_a(2); z := weighted_ab(); }
";

const BAKE_BEFORE: &str = "weighted_a :: #bake_arguments weighted(a = 1);
weighted :: (a: int, b: int) -> int {
    return a * 10 + b;
}

main :: () { x := weighted_a(2); }
";

fn open(defer: bool, program: &str) -> Session {
    let mut s = session();

    s.defer_library_bodies(defer);
    s.open(uri(), 1, program.into()).unwrap();
    s
}

fn check(program: &str) {
    for defer in [false, true] {
        check_in(&open(defer, program), program);
    }
}

fn check_in(s: &Session, program: &str) {
    let use_site = at_offset(program, "a * 10", 0);
    let hover = s.hover(&uri(), use_site).unwrap().expect("hover");

    assert_eq!(hover.contents.value, "a: s64", "{hover:?}");
    let defs = s.definition(&uri(), use_site).unwrap();

    assert_eq!(defs.len(), 1, "{defs:?}");
    assert_eq!(defs[0].range.start, at(program, "a: int"), "{defs:?}");
}

#[test]
fn a_parameter_in_its_own_body_is_not_the_baked_constant() {
    check(BAKE_AFTER);
}

#[test]
fn the_answer_does_not_depend_on_where_the_bake_is() {
    check(BAKE_BEFORE);
}
