//! A local name and its uses are edited together; nothing else is linked.
use jai_language_server::{DocumentUri, Position, Session};
use test_support::lsp::{file_uri, lean_session};

const SOURCE: &str = "\
Thing :: struct { count: int; }
total := 10;
main :: (limit: int) {
    sum := limit + 1;
    other := sum * 2;
    t: Thing;
    t.count = sum;
    print(\"%\", other);
}
";

fn open() -> (Session, DocumentUri) {
    let mut session = lean_session();
    let uri = file_uri("/linked/main.jai");
    session.open(uri.clone(), 1, SOURCE.into()).unwrap();
    (session, uri)
}

fn at(line: u32, character: u32) -> Position {
    Position {
        line,
        character,
    }
}

fn lines(session: &Session, uri: &DocumentUri, position: Position) -> Vec<(u32, u32)> {
    session
        .linked_editing_ranges(uri, position)
        .unwrap()
        .iter()
        .map(|r| (r.start.line, r.start.character))
        .collect()
}

#[test]
fn a_local_and_its_uses_are_linked_from_any_of_them() {
    let (session, uri) = open();
    let expected = vec![(3, 4), (4, 13), (6, 14)];
    assert_eq!(lines(&session, &uri, at(3, 5)), expected);
    assert_eq!(lines(&session, &uri, at(4, 13)), expected);
    assert_eq!(lines(&session, &uri, at(6, 16)), expected);
}

#[test]
fn a_parameter_is_linked_with_its_uses() {
    let (session, uri) = open();
    assert_eq!(lines(&session, &uri, at(2, 9)), [(2, 9), (3, 11)]);
}

#[test]
fn globals_members_and_types_are_not_linked() {
    let (session, uri) = open();
    assert!(lines(&session, &uri, at(1, 2)).is_empty(), "a global");
    assert!(lines(&session, &uri, at(6, 6)).is_empty(), "a member");
    assert!(lines(&session, &uri, at(5, 6)).is_empty(), "a type");
    assert!(lines(&session, &uri, at(0, 1)).is_empty(), "a declaration");
}

#[test]
fn without_a_compile_of_the_text_nothing_is_linked() {
    let (session, uri) = open();
    session.defer_compiles(true);
    let compiles = session.compile_counts().0;
    assert!(lines(&session, &uri, at(3, 5)).is_empty());
    assert_eq!(session.compile_counts().0, compiles, "no compile");
}

#[test]
fn checked_local_references_survive_compiler_eviction_without_recompiling() {
    let (mut session, uri) = open();
    session.defer_compiles(true);
    let others: Vec<_> = (0..8)
        .map(|n| file_uri(&format!("/linked-eviction/other{n}.jai")))
        .collect();
    for other in &others {
        session
            .open(
                other.clone(),
                1,
                format!("{}main :: () {{}}", "// padding\n".repeat(512)),
            )
            .unwrap();
    }
    session.diagnostics(&uri).unwrap();
    let expected = lines(&session, &uri, at(3, 5));
    assert_eq!(expected, [(3, 4), (4, 13), (6, 14)]);
    let references = session.references(&uri, at(3, 5), true).unwrap();
    for other in &others {
        session.diagnostics(other).unwrap();
    }
    let counts = session.compile_counts();
    assert!(
        counts.0 > counts.1 + 1,
        "other checks must exceed compiler retention"
    );
    assert!(
        session.semantic_coverage(&uri).unwrap().is_none(),
        "original compiler must actually be evicted"
    );
    assert_eq!(
        session.references(&uri, at(3, 5), true).unwrap(),
        references
    );
    assert_eq!(lines(&session, &uri, at(3, 5)), expected);
    assert_eq!(
        session.compile_counts(),
        counts,
        "retained navigation never compiles"
    );
}

#[test]
fn a_pending_dependency_edit_retires_exact_linked_edits() {
    use jai_language_server::TextChange;
    let mut session = lean_session();
    let root = file_uri("/linked-dependency/main.jai");
    let loaded = file_uri("/linked-dependency/loaded.jai");
    session
        .open(
            root.clone(),
            1,
            "#load \"loaded.jai\"; main :: () { value := provided; value += 1; }".into(),
        )
        .unwrap();
    session
        .open(loaded.clone(), 1, "provided :: 42;".into())
        .unwrap();
    session.defer_compiles(true);
    session.diagnostics(&root).unwrap();
    let text = session.document_text(&root).unwrap();
    let pos = test_support::lsp::position(text, text.find("value :=").unwrap() + 1);
    assert_eq!(session.linked_editing_ranges(&root, pos).unwrap().len(), 2);
    session
        .change(
            &loaded,
            2,
            &[TextChange {
                range: None,
                range_length: None,
                text: "provided :: \"different type\";".into(),
            }],
        )
        .unwrap();
    let counts = session.compile_counts();
    assert!(
        session
            .linked_editing_ranges(&root, pos)
            .unwrap()
            .is_empty()
    );
    assert_eq!(session.compile_counts(), counts);
}

#[test]
fn trust_limit_and_panic_reset_retire_checked_reference_ownership() {
    for transition in ["trust", "limits", "reset", "workspace"] {
        let (mut session, uri) = open();
        session.defer_compiles(true);
        session.diagnostics(&uri).unwrap();
        assert_eq!(lines(&session, &uri, at(3, 5)).len(), 3);
        assert!(session.reference_record_counts().0 > 0);
        match transition {
            "trust" => session.set_trusted(false),
            "limits" => session.set_evaluation_budget(1024),
            "reset" => session.reset_caches(),
            "workspace" => session.set_workspace_folders(vec!["/different-workspace".into()]),
            _ => unreachable!(),
        }
        assert_eq!(session.reference_record_counts(), (0, 0), "{transition}");
        let counts = session.compile_counts();
        if transition != "workspace" {
            assert!(lines(&session, &uri, at(3, 5)).is_empty(), "{transition}");
        }
        assert_eq!(session.compile_counts(), counts, "{transition}");
    }
}

#[test]
fn close_reopen_never_reuses_the_closed_documents_reference_record() {
    let (mut session, uri) = open();
    session.defer_compiles(true);
    session.diagnostics(&uri).unwrap();
    assert!(session.reference_record_counts().0 > 0);
    session.close(&uri).unwrap();
    assert_eq!(session.reference_record_counts(), (0, 0));
    session.open(uri.clone(), 2, format!("\n{SOURCE}")).unwrap();
    let counts = session.compile_counts();
    assert!(lines(&session, &uri, at(4, 5)).is_empty());
    assert_eq!(session.compile_counts(), counts);
    session.diagnostics(&uri).unwrap();
    assert_eq!(lines(&session, &uri, at(4, 5)), [(4, 4), (5, 13), (7, 14)]);
}
