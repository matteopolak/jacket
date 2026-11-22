//! A hover shows a whole declaration header, even one written over several lines, as one line.
use test_support::lsp::{at, open, uri};

const PROGRAM: &str = "copy :: (
    using source: Bits, // the bits to copy
    extra: s64 = 0,
) -> Bits {
    return source;
}

Pair :: struct (
    A: Type,
    B: Type
) {
    a: A;
    b: B;
}

Bits :: struct { count: s64; }

main :: () {
    b: Bits;
    c := copy(b);
    p: Pair(s64, bool);
}
";

fn hover(marker: &str) -> String {
    let s = open(PROGRAM);
    s.hover(&uri(), at(PROGRAM, marker))
        .unwrap()
        .expect("hover")
        .contents
        .value
}

#[test]
fn a_multi_line_procedure_header_shows_every_parameter() {
    for marker in ["copy ::", "copy(b)"] {
        let text = hover(marker);
        assert!(
            text.contains("copy :: (using source: Bits, extra: s64 = 0) -> Bits"),
            "{text}"
        );
        assert!(
            !text.contains("bits to copy") && !text.contains("return"),
            "{text}"
        );
    }
}

#[test]
fn a_multi_line_polymorphic_struct_shows_every_parameter() {
    let text = hover("Pair ::");
    assert!(text.contains("Pair :: struct (A: Type, B: Type)"), "{text}");
}
