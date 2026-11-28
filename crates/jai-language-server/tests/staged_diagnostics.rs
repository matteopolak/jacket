//! An edit publishes the compiler's diagnostics first and the lints after them, without losing
//! or reordering any. The interrupt state is process-wide, so every test takes `LOCK`.
use std::sync::Mutex;

use jai_language_server::{Diagnostic, DiagnosticCode, DocumentUri, Session, Stage};
use test_support::lsp::{session, uri};

static LOCK: Mutex<()> = Mutex::new(());

const SOURCE: &str = "#import \"Basic\";\nmain :: () {\n    xs: [..] int;\n    array_add(*xs, 1);\n    p := *xs[0];\n    array_add(*xs, 2);\n    print(\"%\\n\", p.*);\n    return;\n    never := 1;\n}\n";

fn open() -> Session {
    let mut s = session();

    s.open(
        DocumentUri::parse("file:///lsp-test/jacket.toml").unwrap(),
        1,
        "[lint.rules]\npointer_lifetime = \"warn\"\n".into(),
    )
    .unwrap();
    s.open(uri(), 1, SOURCE.into()).unwrap();
    s
}

fn is_lint(d: &Diagnostic) -> bool {
    matches!(d.code, DiagnosticCode::Lint(_))
}

fn of_source(all: Vec<(String, i32, Vec<Diagnostic>)>) -> Vec<Diagnostic> {
    all.into_iter()
        .find(|(u, ..)| u == uri().as_str())
        .expect("the document is published")
        .2
}

#[test]
fn the_compile_stage_carries_no_lints_and_the_full_stage_all_of_them() {
    let _guard = LOCK
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);

    jaic::interrupt::request(false);
    let s = open();
    let first = of_source(s.staged_unless_interrupted(Stage::Compile).unwrap());

    assert!(
        first.iter().any(|d| d.code == DiagnosticCode::Unreachable),
        "{first:?}"
    );
    assert!(!first.iter().any(is_lint), "{first:?}");
    let full = of_source(s.staged_unless_interrupted(Stage::Full).unwrap());

    assert!(full.iter().any(is_lint), "{full:?}");
    // Nothing the first stage published is gone, and the order is the one a full check has.
    let kept: Vec<_> = full.iter().filter(|d| !is_lint(d)).cloned().collect();

    assert_eq!(format!("{kept:?}"), format!("{first:?}"));
    assert_eq!(
        format!("{full:?}"),
        format!("{:?}", of_source(s.publications()))
    );
}

/// After an edit, the compile stage shows the lints of the last full check where they are now,
/// so they are not cleared and drawn again; the full stage then has the lints of the new text.
#[test]
fn the_compile_stage_after_an_edit_carries_the_last_lints_moved() {
    let _guard = LOCK
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);

    jaic::interrupt::request(false);
    let mut s = open();
    let full = of_source(s.staged_unless_interrupted(Stage::Full).unwrap());
    let lints: Vec<_> = full.into_iter().filter(is_lint).collect();

    assert!(!lints.is_empty());
    let edit = jai_language_server::TextChange {
        range: None,
        range_length: None,
        text: format!("\n{SOURCE}"),
    };

    s.change(&uri(), 2, &[edit]).unwrap();
    let first = of_source(s.staged_unless_interrupted(Stage::Compile).unwrap());
    let carried: Vec<_> = first.into_iter().filter(is_lint).collect();

    assert_eq!(carried.len(), lints.len(), "{carried:?}");

    for (now, before) in carried.iter().zip(&lints) {
        assert_eq!(now.message, before.message);
        assert_eq!(now.range.start.line, before.range.start.line + 1);
    }

    let full = of_source(s.staged_unless_interrupted(Stage::Full).unwrap());
    let found: Vec<_> = full.into_iter().filter(is_lint).collect();

    assert_eq!(format!("{found:?}"), format!("{carried:?}"));
}
