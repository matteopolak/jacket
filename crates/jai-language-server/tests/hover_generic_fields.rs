//! Hovering a field of a generic struct nothing instantiates still shows the field, not the
//! note about syntax-only declarations.
mod common;
use common::{at, open, uri};

const PROGRAM: &str = "Box :: struct(T: Type) {
    count: int;
    item: T;
    label: string = \"box\";
}

main :: () {}
";

fn hover(name: &str) -> String {
    let s = open(PROGRAM);
    s.hover(&uri(), at(PROGRAM, name))
        .unwrap()
        .expect("hover")
        .contents
        .value
}

#[test]
fn a_field_of_an_unused_generic_struct() {
    for name in ["count: int", "item: T", "label: string"] {
        let text = hover(name);
        assert!(
            !text.contains("Source syntax declaration"),
            "{name}: {text}"
        );
    }
}

#[test]
fn the_field_shows_its_type() {
    assert!(hover("count: int").contains("int"));
    assert!(hover("item: T").contains('T'));
}
