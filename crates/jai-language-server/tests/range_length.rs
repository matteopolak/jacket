//! `rangeLength` on an incremental edit is deprecated and clients compute it differently; the
//! range alone decides what is replaced.
mod common;
use common::{open, uri};
use jai_language_server::{Position, Range, TextChange};

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
