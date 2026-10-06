//! Minimized crashes found by the `lsp` and `lsp_edits` fuzz targets (fuzz/). Each request must
//! answer or fail with an error, never panic.
use jai_language_server::{DocumentUri, Environment, Limits, Position, Session};
use std::path::PathBuf;

fn open(text: &str) -> (Session, DocumentUri) {
    open_in(Session::new(Limits::default()), text)
}

/// [`open`] in a session that type-checks against the repository's stdlib.
fn open_checked(text: &str) -> (Session, DocumentUri) {
    let stdlib = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../stdlib");
    let environment = Environment {
        fs: std::rc::Rc::new(jaic::sema::NativeFs),
        options: Box::new(move |_| {
            let mut options = jaic::sema::Options::host();
            options.import_paths = vec![stdlib.clone()];
            options.preload = Some(stdlib.join("prelude/module.jai"));
            options
        }),
    };
    open_in(
        Session::with_environment(Limits::default(), environment),
        text,
    )
}

fn open_in(mut session: Session, text: &str) -> (Session, DocumentUri) {
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

#[test]
fn queries_on_a_builtin_procedure_name() {
    // `type_info(` while typing: signature help resolved the builtin `type_info` as if it were
    // a declaration, which the resolver treated as unreachable.
    // The text does not parse, so signature help looks the callee up by name.
    let text = "S :: struct { a: s32; }\nmain :> () { x := type_info.y; info := type_info(S";
    let (session, uri) = open_checked(text);
    for (line, character) in [(1, 27), (1, 28), (1, 49), (1, 50)] {
        let position = Position {
            line,
            character,
        };
        assert!(session.signature_help(&uri, position).is_ok());
        assert!(session.completion(&uri, position).is_ok());
        let _ = session.hover(&uri, position);
        let _ = session.definition(&uri, position);
    }
}
