//! An edit publishes the compiler's diagnostics first and the lints after them, without losing
//! or reordering any. The interrupt state is process-wide, so every test takes `LOCK`.
use jai_language_server::{Diagnostic, DiagnosticCode, DocumentUri, Session, Stage};
use std::sync::Mutex;
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
