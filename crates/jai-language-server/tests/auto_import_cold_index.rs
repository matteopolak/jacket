//! A request never builds the auto-import module index: a session that defers compiles (the
//! transport's mode) offers no auto imports, and asks again, until the idle warm-up has read it.
use std::path::Path;

use jai_language_server::{CompletionList, Limits, Session};
use test_support::lsp::{end_of, modules_environment, project, uri_of};

fn session() -> Session {
    let s = Session::with_environment(
        Limits::default(),
        modules_environment(Some(jaic::target::TargetOs::Linux)),
    );

    s.defer_compiles(true);
    s
}

fn complete(s: &Session, path: &Path, text: &str, marker: &str) -> CompletionList {
    s.completion(&uri_of(path), end_of(text, marker)).unwrap()
}

fn auto_imports(list: &CompletionList) -> usize {
    list.items
        .iter()
        .filter(|i| !i.additional_edits.is_empty())
        .count()
}

#[test]
fn auto_imports_wait_for_the_warm_up() {
    let dir = project(
        "cold-index",
        &[
            ("main.jai", "main :: () { meas }"),
            (
                "modules/Facade/module.jai",
                "measure :: (n: int) -> int { return n; }",
            ),
        ],
    );
    let path = dir.join("main.jai");
    let text = "main :: () { meas }";
    let mut s = session();

    s.open(uri_of(&path), 1, text.into()).unwrap();
    let cold = complete(&s, &path, text, "meas");

    assert_eq!(auto_imports(&cold), 0, "{cold:?}");
    assert!(cold.is_incomplete, "the client asks again");
    s.warm_auto_import();
    let warm = complete(&s, &path, text, "meas");

    assert!(auto_imports(&warm) > 0, "{warm:?}");
}

/// The stdlib's modules are shared between sessions, so a session's own scans of its files are
/// not kept once the module list is made.
#[test]
fn the_stdlib_scans_do_not_stay_in_the_index() {
    let dir = project(
        "stdlib-scans",
        &[("main.jai", "#import \"Basic\";\nmain :: () { meas }")],
    );
    let path = dir.join("main.jai");
    let text = "#import \"Basic\";\nmain :: () { meas }";
    let mut s = Session::with_environment(Limits::default(), modules_environment(None));

    s.open(uri_of(&path), 1, text.into()).unwrap();
    s.warm_auto_import();
    let _ = complete(&s, &path, text, "meas");

    assert_eq!(s.retained_scans_under(&test_support::lsp::stdlib()), 0);
}
