//! `rangeLength` on an incremental edit is deprecated and clients compute it differently; the
//! range alone decides what is replaced.
use jai_language_server::{Position, Range, TextChange};
use test_support::lsp::{open, uri};

#[test]
fn a_range_length_that_disagrees_with_the_range_is_ignored() {
    let mut s = open("main :: () {}\n");
    let range = Range {
        start: Position {
            line: 0,
            character: 0,
        },
        end: Position {
            line: 0,
            character: 4,
        },
    };
    s.change(
        &uri(),
        2,
        &[TextChange {
            range: Some(range),
            range_length: Some(99),
            text: "go".into(),
        }],
    )
    .unwrap();
    assert_eq!(s.document_text(&uri()).unwrap(), "go :: () {}\n");
}
