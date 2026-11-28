//! Hovering a field of a generic struct nothing instantiates still shows the field, not the
//! note about syntax-only declarations.
use test_support::lsp::{at, open, position, uri};

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

#[test]
fn the_type_parameter_of_a_generic_struct_has_a_hover() {
    let text = hover("T: Type");

    assert!(text.contains('T'), "{text}");
    assert!(!text.contains("Source syntax declaration"), "{text}");
}

#[test]
fn the_type_parameter_has_a_hover_after_every_other_name_was_asked() {
    let s = open(PROGRAM);

    for offset in (0..PROGRAM.len()).filter(|i| PROGRAM.is_char_boundary(*i)) {
        let _ = s.hover(&uri(), position(PROGRAM, offset));
    }

    let hover = s
        .hover(&uri(), at(PROGRAM, "T: Type"))
        .unwrap()
        .expect("hover on T");

    assert!(
        hover.contents.value.contains('T'),
        "{}",
        hover.contents.value
    );
}
