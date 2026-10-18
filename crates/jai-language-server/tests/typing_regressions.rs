//! Cases the typing-session checker found (docs/tools/lsp-fuzzing.md), reduced by hand: answers
//! while someone types must not depend on which requests came before, on a half-typed neighbour,
//! or on a lexical edit that has since been completed.
#[allow(dead_code)]
#[path = "../examples/support/typing_files.rs"]
mod typing_files;

use jai_language_server::{
    DocumentUri, Limits, Position, Range, SemanticTokenKind, Session, TextChange,
};
use std::path::Path;

fn session() -> Session {
    let stdlib = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../stdlib")
        .canonicalize()
        .unwrap();
    Session::with_environment(Limits::default(), typing_files::native_environment(stdlib))
}

fn uri() -> DocumentUri {
    DocumentUri::parse("file:///workspace/main.jai").unwrap()
}

fn open(text: &str) -> Session {
    let mut session = session();
    session.open(uri(), 1, text.into()).unwrap();
    session
}

fn position_of(text: &str, byte: usize) -> Position {
    let before = &text[..byte];
    Position {
        line: before.matches('\n').count() as u32,
        character: before[before.rfind('\n').map_or(0, |i| i + 1)..]
            .encode_utf16()
            .count() as u32,
    }
}

/// Insert `typed` at byte `at` of the document as an editor would: one character per change.
fn type_in(session: &mut Session, text: &mut String, at: usize, typed: &str, version: &mut i32) {
    for (offset, c) in typed.char_indices() {
        let p = position_of(text, at + offset);
        *version += 1;
        session
            .change(
                &uri(),
                *version,
                &[TextChange {
                    range: Some(Range {
                        start: p,
                        end: p,
                    }),
                    range_length: None,
                    text: c.to_string(),
                }],
            )
            .unwrap();
        text.insert(at + offset, c);
        // Requests between keystrokes, as an editor makes them.
        let _ = session.diagnostics(&uri());
        let _ = session.semantic_tokens(&uri());
    }
}

fn kind_at(session: &Session, line: u32, character: u32) -> Option<SemanticTokenKind> {
    session
        .semantic_tokens(&uri())
        .unwrap()
        .into_iter()
        .find(|t| t.position.line == line && t.position.character == character)
        .map(|t| t.kind)
}

fn messages(session: &Session) -> Vec<String> {
    session
        .diagnostics(&uri())
        .unwrap()
        .into_iter()
        .map(|d| format!("{:?} {}", d.range, d.message))
        .collect()
}

fn on_big_stack(work: impl FnOnce() + Send + 'static) {
    std::thread::Builder::new()
        .stack_size(1 << 29)
        .spawn(work)
        .unwrap()
        .join()
        .unwrap();
}

#[test]
fn a_string_closed_after_typing_leaves_no_lexical_damage() {
    on_big_stack(|| {
        let mut text = "main :: () {\n    \n}\n".to_string();
        let mut session = open(&text);
        let mut version = 1;
        let at = text.find("    \n").unwrap() + 4;
        type_in(&mut session, &mut text, at, "x := \"abc", &mut version);
        let close = at + "x := \"abc".len();
        type_in(&mut session, &mut text, close, "\";", &mut version);
        let fresh = open(&text);
        assert_eq!(messages(&session), messages(&fresh), "{text}");
        assert!(
            !messages(&session)
                .iter()
                .any(|m| m.contains("Incomplete lexical edit")),
            "{:?}",
            messages(&session)
        );
    });
}

#[test]
fn an_unfinished_cast_at_the_end_is_reported_once() {
    on_big_stack(|| {
        let session = open("x := cast(\n {\n __t");
        let all = messages(&session);
        let mut unique = all.clone();
        unique.sort();
        unique.dedup();
        assert_eq!(all.len(), unique.len(), "{all:#?}");
    });
}

#[test]
fn a_callee_has_the_same_color_whether_or_not_completion_ran() {
    on_big_stack(|| {
        let text = "#import \"Basic\";\nmain :: () {\n    print(\"hi\\n\");\n}\n";
        let call = position_of(text, text.find("print").unwrap());
        let open_paren = position_of(text, text.find("print(").unwrap() + "print(".len());
        let plain = open(text);
        let before = kind_at(&plain, call.line, call.character);
        let asked = open(text);
        let _ = asked.signature_help(&uri(), open_paren);
        let _ = asked.completion(&uri(), open_paren);
        assert_eq!(kind_at(&asked, call.line, call.character), before);
        assert!(before.is_some());
    });
}

#[test]
fn a_module_alias_is_a_namespace_before_and_after_it_is_hovered() {
    on_big_stack(|| {
        let text = "B :: #import \"Basic\";\nmain :: () {\n    B.print(\"hi\\n\");\n}\n\n#run {\n    B.print(\"run\\n\");\n}\n";
        let session = open(text);
        for (line, character) in [(2, 4), (6, 4)] {
            let first = kind_at(&session, line, character);
            let _ = session.hover(
                &uri(),
                Position {
                    line,
                    character,
                },
            );
            let again = kind_at(&session, line, character);
            assert_eq!(first, again, "line {line}");
            assert_eq!(first, Some(SemanticTokenKind::Namespace), "line {line}");
        }
        // Reading the tokens twice gives the same answer too.
        assert_eq!(
            session.semantic_tokens(&uri()).unwrap(),
            session.semantic_tokens(&uri()).unwrap()
        );
    });
}

#[test]
fn a_statement_at_file_scope_does_not_hide_the_directives_after_it() {
    on_big_stack(|| {
        let text = "P :: #import \"Preload\";\n#assert P.Allocator == Allocator;\n\nfor 0..3 {}\n";
        let session = open(text);
        assert_eq!(
            kind_at(&session, 1, 8),
            Some(SemanticTokenKind::Namespace),
            "the alias in the assertion"
        );
        assert_eq!(kind_at(&session, 1, 10), Some(SemanticTokenKind::Type));
        assert!(
            messages(&session)
                .iter()
                .any(|m| m.contains("not allowed at file scope")),
            "{:?}",
            messages(&session)
        );
    });
}
