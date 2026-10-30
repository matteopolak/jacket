//! Hover and definition on the target of a compound assignment (`p.x += 1`).
use test_support::lsp::{at, open, uri};

const PROGRAM: &str = "Counter :: struct { next: int; }

bump :: (counter: *Counter) {
    counter.next += 1;
}

main :: () {
    c: Counter;
    bump(*c);
}
";

/// Through a pointer the target is evaluated once, into a hidden local; that local is the
/// compiler's, so the name the user wrote is what hover and definition describe.
#[test]
fn the_target_through_a_pointer_is_the_users_name() {
    let s = open(PROGRAM);
    let use_site = at(PROGRAM, "counter.next +=");
    let hover = s.hover(&uri(), use_site).unwrap().expect("hover");
    assert!(hover.contents.value.contains("counter"), "{hover:?}");
    assert!(!hover.contents.value.contains('\0'), "{hover:?}");
    assert!(
        hover.range.end.character > hover.range.start.character,
        "{hover:?}"
    );
    let definition = s.definition(&uri(), use_site).unwrap();
    assert_eq!(definition.len(), 1, "{definition:?}");
    assert_eq!(definition[0].range.start, at(PROGRAM, "counter: *Counter"));
}
