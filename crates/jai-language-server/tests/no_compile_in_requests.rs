//! With compiles deferred, no request makes one: after an edit they answer from the last
//! compile (positions moved past the edit) or leave out what needs a fresh one, and the compile
//! is made by `warm_pending` while input is quiet.
use jai_language_server::{DocumentUri, Position, Range, Session, TextChange};
use test_support::lsp::{file_uri, lean_session};

const SOURCE: &str = "\
helper :: (first: int, second: int) -> int { return first + second; }
main :: () {
    total := helper(1, 2);
    tot
}
";

/// A session that has compiled `SOURCE`, then had a blank line typed at the start of the file
/// (Enter at a line start): no compile knows the text, and every offset has moved.
fn edited(name: &str) -> (Session, DocumentUri, String) {
    edited_to(name, &format!("\n{SOURCE}"))
}

fn edited_to(name: &str, text: &str) -> (Session, DocumentUri, String) {
    let mut session = lean_session();
    session.defer_compiles(true);
    let uri = file_uri(&format!("/no-compile/{name}.jai"));
    session.open(uri.clone(), 1, SOURCE.into()).unwrap();
    session.publications();
    let edit = TextChange {
        range: None,
        range_length: None,
        text: text.into(),
    };
    session.change(&uri, 2, &[edit]).unwrap();
    (session, uri, text.into())
}

fn at(line: u32, character: u32) -> Position {
    Position {
        line,
        character,
    }
}

fn whole(text: &str) -> Range {
    Range {
        start: at(0, 0),
        end: at(text.lines().count() as u32 + 1, 0),
    }
}

#[test]
fn completion_after_enter_and_a_key_answers_from_the_last_compile() {
    let (session, uri, _) = edited("completion");
    let compiles = session.compile_counts().0;
    let list = session.completion(&uri, at(4, 7)).unwrap();
    assert_eq!(session.compile_counts().0, compiles, "no compile");
    assert!(list.items.iter().any(|i| i.label == "total"), "{list:?}");
    assert!(list.is_incomplete, "the names may be out of date");
    assert!(session.compile_pending());
}

#[test]
fn hover_moves_to_where_the_word_is_now() {
    let (session, uri, _) = edited("hover");
    let compiles = session.compile_counts().0;
    // `helper` in `total := helper(1, 2);`, one line lower than compiled.
    let hover = session.hover(&uri, at(3, 15)).unwrap().expect("a hover");
    assert_eq!(session.compile_counts().0, compiles, "no compile");
    assert_eq!(hover.range.start, at(3, 13));
    assert_eq!(hover.range.end, at(3, 19));
    assert!(hover.contents.value.contains("helper"), "{hover:?}");
}

#[test]
fn definition_moves_to_where_the_declaration_is_now() {
    let (session, uri, _) = edited("definition");
    let compiles = session.compile_counts().0;
    let found = session.definition(&uri, at(3, 15)).unwrap();
    assert_eq!(session.compile_counts().0, compiles, "no compile");
    assert_eq!(found.len(), 1);
    assert_eq!(
        found[0].range.start.line, 1,
        "the blank line pushed it down"
    );
    assert_eq!(found[0].range.start.character, 0);
}

#[test]
fn signature_help_while_typing_a_call_uses_the_last_compile() {
    let text = SOURCE.replace("helper(1, 2);", "helper(1, ");
    let (session, uri, _) = edited_to("signature", &text);
    let compiles = session.compile_counts().0;
    let help = session
        .signature_help(&uri, at(2, 22))
        .unwrap()
        .expect("signature help");
    assert_eq!(session.compile_counts().0, compiles, "no compile");
    assert!(help.signatures[0].label.contains("helper"), "{help:?}");
    assert_eq!(help.active_parameter, 1);
}

#[test]
fn every_other_request_answers_without_a_compile() {
    let (session, uri, text) = edited("others");
    let compiles = session.compile_counts().0;
    let range = whole(&text);
    let _ = session.semantic_tokens(&uri).unwrap();
    let _ = session.inlay_hints(&uri, range).unwrap();
    let _ = session.document_symbols(&uri).unwrap();
    let _ = session.code_actions(&uri, range).unwrap();
    let _ = session.references(&uri, at(3, 15), true).unwrap();
    let _ = session.document_highlights(&uri, at(3, 15)).unwrap();
    let _ = session.type_definition(&uri, at(3, 15)).unwrap();
    let _ = session.prepare_call_hierarchy(&uri, at(1, 2)).unwrap();
    let _ = session.code_lenses(&uri).unwrap();
    let _ = session.selection_ranges(&uri, &[at(3, 15)]).unwrap();
    let _ = session.folding_ranges(&uri).unwrap();
    let _ = session.workspace_symbols("help");
    assert_eq!(session.compile_counts().0, compiles, "no compile");
}

#[test]
fn the_compile_is_made_once_input_is_quiet_and_requests_then_use_it() {
    let (session, uri, _) = edited("warm");
    session.hover(&uri, at(3, 15)).unwrap();
    session.inlay_hints(&uri, whole(SOURCE)).unwrap();
    assert!(session.compile_pending());
    let compiles = session.compile_counts().0;
    assert_eq!(session.warm_pending(), Some(true));
    assert_eq!(session.compile_counts().0, compiles + 1);
    assert!(!session.compile_pending());
    let list = session.completion(&uri, at(4, 7)).unwrap();
    assert_eq!(session.compile_counts().0, compiles + 1, "from the cache");
    assert!(list.items.iter().any(|i| i.label == "total"));
}

#[test]
fn a_rename_compiles_because_its_edits_must_be_exact() {
    let (session, uri, _) = edited("rename");
    let compiles = session.compile_counts().0;
    let edits = session.rename(&uri, at(3, 15), "assist").unwrap();
    assert!(edits.is_some());
    assert!(session.compile_counts().0 > compiles);
}

#[test]
fn without_deferring_requests_compile_as_before() {
    let mut session = lean_session();
    let uri = file_uri("/no-compile/plain.jai");
    session.open(uri.clone(), 1, SOURCE.into()).unwrap();
    session.publications();
    let compiles = session.compile_counts().0;
    let edit = TextChange {
        range: None,
        range_length: None,
        text: format!("\n{SOURCE}"),
    };
    session.change(&uri, 2, &[edit]).unwrap();
    let list = session.completion(&uri, at(4, 7)).unwrap();
    assert!(session.compile_counts().0 > compiles);
    assert!(!list.is_incomplete);
}
