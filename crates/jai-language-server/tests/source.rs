use jai_language_server::{
    DiagnosticCode, DiagnosticSeverity, DocumentUri, Error, Limits, Position, Range,
    SemanticTokenKind, Session, SymbolKind, TextChange,
};
use jai_source::SourceProvider;
use std::path::Path;
fn uri(name: &str) -> DocumentUri {
    DocumentUri::parse(&format!("file:///workspace/{name}")).unwrap()
}
fn at(text: &str, name: &str) -> Position {
    let byte = text.rfind(name).unwrap();
    let prefix = &text[..byte];
    Position {
        line: prefix.bytes().filter(|b| *b == b'\n').count() as u32,
        character: prefix.rsplit('\n').next().unwrap().encode_utf16().count() as u32,
    }
}
fn edit(start: Position, end: Position, text: &str) -> TextChange {
    TextChange {
        range: Some(Range {
            start,
            end,
        }),
        range_length: None,
        text: text.into(),
    }
}

#[test]
fn utf16_crlf_surrogates_and_versioned_edits_are_atomic() {
    let mut session = Session::new(Limits::default());
    let name = uri("main.jai");
    let original = "// 🦀\r\nvalue :: 1;\r\nmain :: () -> int {return value;}";
    session.open(name.clone(), 1, original.into()).unwrap();
    let pinned = session.source_snapshot();
    let invalid = edit(
        Position {
            line: 0,
            character: 4,
        },
        Position {
            line: 0,
            character: 5,
        },
        "x",
    );
    assert!(matches!(
        session.change(&name, 2, &[invalid]),
        Err(Error::Position(_))
    ));
    assert_eq!(session.document_text(&name).unwrap(), original);
    assert_eq!(session.version(&name).unwrap(), 1);
    let change = TextChange {
        range: Some(Range {
            start: Position {
                line: 0,
                character: 3,
            },
            end: Position {
                line: 0,
                character: 5,
            },
        }),
        range_length: Some(2),
        text: "é".into(),
    };
    session.change(&name, 2, &[change]).unwrap();
    assert!(
        session
            .document_text(&name)
            .unwrap()
            .starts_with("// é\r\n")
    );
    assert!(matches!(
        session.change(&name, 2, &[]),
        Err(Error::StaleVersion)
    ));
    assert_eq!(
        pinned.read(Path::new("/workspace/main.jai")).unwrap(),
        original.as_bytes()
    );
    assert!(pinned.read(Path::new("/etc/passwd")).is_err());
    assert!(
        pinned
            .retain_decoded_text(Path::new("/workspace/main.jai"), "different")
            .is_err()
    );
}
#[test]
fn a_late_invalid_edit_rolls_back_the_entire_change_batch() {
    let mut session = Session::new(Limits::default());
    let name = uri("main.jai");
    let text = "value :: 1;";
    session.open(name.clone(), 1, text.into()).unwrap();
    let changes = [
        edit(
            Position {
                line: 0,
                character: 9,
            },
            Position {
                line: 0,
                character: 10,
            },
            "2",
        ),
        edit(
            Position {
                line: 99,
                character: 0,
            },
            Position {
                line: 99,
                character: 0,
            },
            "x",
        ),
    ];
    assert!(session.change(&name, 2, &changes).is_err());
    assert_eq!(session.document_text(&name).unwrap(), text);
    assert_eq!(session.version(&name).unwrap(), 1);
}
#[test]
fn authentic_load_declarations_drive_multifile_navigation() {
    let mut session = Session::new(Limits::default());
    let main = uri("main.jai");
    let helper = uri("lib/answer.jai");
    let text = "#load \"lib/answer.jai\";\nmain :: () -> int {return answer();}";
    session.open(main.clone(), 1, text.into()).unwrap();
    assert!(
        session
            .diagnostics(&main)
            .unwrap()
            .iter()
            .any(|d| d.code == DiagnosticCode::Source && d.severity == DiagnosticSeverity::Warning)
    );
    session
        .open(helper.clone(), 1, "answer :: () -> int {return 42;}".into())
        .unwrap();
    assert!(session.diagnostics(&main).unwrap().is_empty());
    let defs = session.definition(&main, at(text, "answer")).unwrap();
    assert_eq!(defs.len(), 1);
    assert_eq!(defs[0].uri, helper.as_str());
    assert_eq!(
        defs[0].range.start,
        Position {
            line: 0,
            character: 0
        }
    );
    assert!(
        session
            .hover(&main, at(text, "answer"))
            .unwrap()
            .unwrap()
            .contents
            .value
            .contains("return 42")
    );
    assert!(
        session
            .completion(&main, at(text, "answer"))
            .unwrap()
            .items
            .iter()
            .any(|item| item.label == "answer")
    );
    session.close(&helper).unwrap();
    assert!(
        session
            .definition(&main, at(text, "answer"))
            .unwrap()
            .is_empty()
    );
}
#[test]
fn local_shadowing_and_closed_lookup_do_not_guess_global_or_member_types() {
    let mut session = Session::new(Limits::default());
    let main = uri("main.jai");
    let text = "value :: 1; main :: () -> int {value := 2; return value;}";
    session.open(main.clone(), 1, text.into()).unwrap();
    let defs = session.definition(&main, at(text, "value")).unwrap();
    assert_eq!(defs.len(), 1);
    assert!(defs[0].range.start.character > 10);
    session
        .open(uri("unrelated.jai"), 1, "other :: 7;".into())
        .unwrap();
    let updated = "value :: 1; main :: () -> int {return other;}";
    session
        .change(
            &main,
            2,
            &[TextChange {
                range: None,
                range_length: None,
                text: updated.into(),
            }],
        )
        .unwrap();
    assert!(
        session
            .definition(&main, at(updated, "other"))
            .unwrap()
            .is_empty()
    );
    let member = "value :: 1; main :: () -> int {return object.value;}";
    session
        .change(
            &main,
            3,
            &[TextChange {
                range: None,
                range_length: None,
                text: member.into(),
            }],
        )
        .unwrap();
    assert!(
        session
            .definition(&main, at(member, "value"))
            .unwrap()
            .is_empty()
    );
}
#[test]
fn incomplete_source_keeps_real_diagnostics_and_tokens_without_running_code() {
    let mut session = Session::new(Limits::default());
    let name = uri("main.jai");
    session.open(name.clone(), 1, "main :: (".into()).unwrap();
    assert!(!session.diagnostics(&name).unwrap().is_empty());
    assert!(!session.semantic_tokens(&name).unwrap().is_empty());
    assert!(session.document_symbols(&name).unwrap().is_empty());
    let never_execute = "#run {while true {}} main :: () -> int {return 42;}";
    session
        .change(
            &name,
            2,
            &[TextChange {
                range: None,
                range_length: None,
                text: never_execute.into(),
            }],
        )
        .unwrap();
    assert!(session.diagnostics(&name).unwrap().is_empty());
    assert_eq!(session.document_symbols(&name).unwrap()[0].name, "main");
}
#[test]
fn resource_admission_and_source_symbol_ranges_are_bounded() {
    let mut session = Session::new(Limits {
        documents: 1,
        document_bytes: 128,
        ..Limits::default()
    });
    let name = uri("main.jai");
    let text = "Thing :: struct {field: int;}; value :: \"🦀\";";
    session.open(name.clone(), 1, text.into()).unwrap();
    let rows = session.document_symbols(&name).unwrap();
    assert_eq!(rows[0].name, "Thing");
    assert_eq!(rows[0].kind, SymbolKind::Struct);
    assert_eq!(rows[0].children[0].name, "field");
    assert_eq!(rows[0].children[0].kind, SymbolKind::Property);
    assert!(matches!(
        session.open(uri("extra.jai"), 1, "x::1;".into()),
        Err(Error::Limit(_))
    ));
    assert!(matches!(
        session.change(
            &name,
            2,
            &[TextChange {
                range: None,
                range_length: None,
                text: "x".repeat(129)
            }]
        ),
        Err(Error::Limit(_))
    ));
    let data = session.semantic_tokens(&name).unwrap();
    assert!(
        data.iter()
            .any(|entry| entry.kind == SemanticTokenKind::String && entry.length == 4),
        "a quoted supplementary Unicode character occupies four UTF-16 units"
    );
    assert!(DocumentUri::parse("file://remote/etc/passwd").is_err());
    assert!(DocumentUri::parse("file:///../escape.jai").is_err());
    assert!(DocumentUri::parse("file:///workspace/bad%00name.jai").is_err());
}
