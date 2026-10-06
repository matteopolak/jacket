//! Minimized crashes found by the `lsp` and `lsp_edits` fuzz targets (fuzz/). Each request must answer or fail
//! with an error, never panic.
use jai_language_server::{DocumentUri, Limits, Position, Session};

fn open(text: &str) -> (Session, DocumentUri) {
    let mut session = Session::new(Limits::default());
    let uri = DocumentUri::parse("file:///workspace/main.jai").unwrap();
    session.open(uri.clone(), 1, text.into()).unwrap();
    (session, uri)
}

#[test]
fn completion_after_a_multibyte_separator() {
    // The word being completed starts after a no-break space (two bytes in UTF-8).
    let (session, uri) = open("x :: 1;\n\u{a0}ab");
    let position = Position {
        line: 1,
        character: 3,
    };
    assert!(session.completion(&uri, position).is_ok());
}

#[test]
fn signature_help_for_a_call_at_the_start_of_the_document() {
    // The callee begins at byte 0, so its start must not be computed by subtracting first.
    for text in ["f(1, ", "a.b("] {
        let (session, uri) = open(text);
        let position = Position {
            line: 0,
            character: text.len() as u32,
        };
        assert!(session.signature_help(&uri, position).is_ok());
    }
}
