//! A check that input interrupts leaves nothing behind, and requests are never interrupted.
//!
//! The interrupt state is process-wide, so every test here takes `LOCK`, and nothing else
//! runs in this binary.
use std::sync::Mutex;
use test_support::lsp::{file_uri, lean_session};

static LOCK: Mutex<()> = Mutex::new(());

const SOURCE: &str = "\
helper :: (a: int) -> int { return a + 1; }
other :: (b: int) -> int { return helper(b) * 2; }
main :: () {
    bad: int = \"text\";
    total := other(helper(2));
    tot
}
";

fn lock() -> std::sync::MutexGuard<'static, ()> {
    LOCK.lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

fn reset() {
    jaic::interrupt::request(false);
    jaic::interrupt::stop_after(0);
    let _ = jaic::interrupt::disarm();
}

fn shown(publications: &[(String, i32, Vec<jai_language_server::Diagnostic>)]) -> String {
    format!("{publications:?}")
}

#[test]
fn without_input_the_result_is_the_same() {
    let _guard = lock();
    reset();
    let uri = file_uri("/interrupt/same.jai");
    let mut stopped = lean_session();
    stopped.open(uri.clone(), 1, SOURCE.into()).unwrap();
    let mut plain = lean_session();
    plain.open(uri, 1, SOURCE.into()).unwrap();
    let expected = plain.publications();
    assert!(
        expected.iter().any(|(_, _, d)| !d.is_empty()),
        "the program has an error to publish"
    );
    let got = stopped
        .publications_unless_interrupted()
        .expect("nothing asked it to stop");
    assert_eq!(shown(&got), shown(&expected));
}

#[test]
fn a_check_stopped_in_the_middle_publishes_nothing_and_poisons_nothing() {
    let _guard = lock();
    reset();
    let uri = file_uri("/interrupt/middle.jai");
    let mut session = lean_session();
    session.open(uri.clone(), 1, SOURCE.into()).unwrap();
    // Stop at the third body: some are checked already, others are not.
    jaic::interrupt::stop_after(3);
    assert!(session.publications_unless_interrupted().is_none());
    reset();
    assert_eq!(
        session.compile_counts().1,
        0,
        "the partial compile is not kept"
    );
    // The next check, undisturbed, is the one a fresh session gives.
    let again = session.publications();
    let mut fresh = lean_session();
    fresh.open(uri, 1, SOURCE.into()).unwrap();
    assert_eq!(shown(&again), shown(&fresh.publications()));
    assert!(again.iter().any(|(_, _, d)| !d.is_empty()));
}

#[test]
fn a_stopped_check_keeps_the_compile_requests_use() {
    let _guard = lock();
    reset();
    let uri = file_uri("/interrupt/kept.jai");
    let mut session = lean_session();
    session.open(uri.clone(), 1, SOURCE.into()).unwrap();
    session.publications();
    let (compiles, alive) = session.compile_counts();
    // An edit changes the program; its check is stopped part way.
    let edit = jai_language_server::TextChange {
        range: None,
        range_length: None,
        text: SOURCE.replace("total := ", "total2 := "),
    };
    session.change(&uri, 2, &[edit]).unwrap();
    jaic::interrupt::stop_after(2);
    assert!(session.publications_unless_interrupted().is_none());
    reset();
    assert_eq!(session.compile_counts().1, alive, "the older compile stays");
    assert!(session.compile_counts().0 > compiles);
}

#[test]
fn requests_are_never_interrupted() {
    let _guard = lock();
    reset();
    let uri = file_uri("/interrupt/requests.jai");
    let mut session = lean_session();
    session.open(uri.clone(), 1, SOURCE.into()).unwrap();
    // Input is waiting and a stop is due, but these are not armed: they compile and answer.
    jaic::interrupt::request(true);
    jaic::interrupt::stop_after(1);
    let completion = session
        .completion(
            &uri,
            jai_language_server::Position {
                line: 5,
                character: 7,
            },
        )
        .unwrap();
    assert!(
        completion.items.iter().any(|i| i.label == "total"),
        "completion answered from a full compile"
    );
    let hover = session
        .hover(
            &uri,
            jai_language_server::Position {
                line: 4,
                character: 5,
            },
        )
        .unwrap();
    assert!(hover.is_some(), "hover answered");
    assert!(session.compile_counts().1 >= 1);
    reset();
}

#[test]
fn interrupted_warm_up_retries_without_caching_partial_modules() {
    let _guard = lock();
    reset();
    let dir = test_support::lsp::project(
        "interrupt-warm",
        &[
            ("main.jai", "main :: () { unique_warm }"),
            ("modules/Warm/module.jai", "unique_warm :: 42;"),
        ],
    );
    let uri = test_support::lsp::uri_of(&dir.join("main.jai"));
    let mut session = jai_language_server::Session::with_environment(
        jai_language_server::Limits::default(),
        test_support::lsp::modules_environment(None),
    );
    session
        .open(uri.clone(), 1, "main :: () { unique_warm }".into())
        .unwrap();
    jaic::interrupt::stop_after(3);
    assert!(!session.warm_auto_import_unless_interrupted());
    reset();
    assert!(session.warm_auto_import_unless_interrupted());
    let result = session
        .completion(
            &uri,
            jai_language_server::Position {
                line: 0,
                character: 24,
            },
        )
        .unwrap();
    assert!(
        result
            .items
            .iter()
            .any(|item| item.label == "unique_warm" && !item.additional_edits.is_empty())
    );
}

#[test]
fn interrupted_lints_are_not_cached_and_retry_matches_fresh() {
    let _guard = lock();
    reset();
    let uri = file_uri("/interrupt/lints.jai");
    let text = "main :: () { unused := 42; }";
    let mut session = lean_session();
    session.open(uri.clone(), 1, text.into()).unwrap();
    // Populate only the semantic compile, leaving the lint pass cold.
    session
        .hover(
            &uri,
            jai_language_server::Position {
                line: 0,
                character: 14,
            },
        )
        .unwrap();
    let compiles = session.compile_counts().0;
    jaic::interrupt::stop_after(2);
    assert!(session.publications_unless_interrupted().is_none());
    assert_eq!(
        session.compile_counts().0,
        compiles,
        "the stop was in lint, not compile"
    );
    reset();
    let actual = session.publications();
    let mut fresh = lean_session();
    fresh.open(uri, 1, text.into()).unwrap();
    assert_eq!(shown(&actual), shown(&fresh.publications()));
    assert!(actual.iter().any(|(_, _, ds)| {
        ds.iter()
            .any(|d| d.code == jai_language_server::DiagnosticCode::Lint("unused_variable"))
    }));
}

#[test]
fn interrupted_warm_up_keeps_completed_scans_but_retries_the_interrupted_file() {
    use jaic::sema::{FileSystem, NativeFs};
    use std::cell::{Cell, RefCell};
    use std::path::{Path, PathBuf};
    use std::rc::Rc;
    struct StopOnB {
        stop: Cell<bool>,
        reads: RefCell<Vec<PathBuf>>,
    }
    impl FileSystem for StopOnB {
        fn read(&self, path: &Path) -> Option<Vec<u8>> {
            self.reads.borrow_mut().push(path.to_path_buf());
            if path.ends_with("B/module.jai") && self.stop.replace(false) {
                jaic::interrupt::request(true);
            }
            NativeFs.read(path)
        }
        fn is_file(&self, path: &Path) -> bool {
            NativeFs.is_file(path)
        }
        fn is_dir(&self, path: &Path) -> bool {
            NativeFs.is_dir(path)
        }
        fn canonical(&self, path: &Path) -> PathBuf {
            NativeFs.canonical(path)
        }
        fn list_dir(&self, path: &Path) -> Vec<(String, bool)> {
            NativeFs.list_dir(path)
        }
        fn stamp(&self, path: &Path) -> Option<u64> {
            NativeFs.stamp(path)
        }
    }
    let _guard = lock();
    reset();
    let dir = test_support::lsp::project(
        "warm-progress",
        &[
            ("main.jai", "main :: () {}"),
            ("stdlib/A/module.jai", "alpha_symbol :: 1;"),
            ("stdlib/B/module.jai", "bravo_symbol :: 2;"),
        ],
    );
    let fs = Rc::new(StopOnB {
        stop: Cell::new(true),
        reads: RefCell::default(),
    });
    let mut environment = test_support::lsp::environment_at(dir.join("stdlib"), None);
    environment.fs = fs.clone();
    let mut session = jai_language_server::Session::with_environment(
        jai_language_server::Limits::default(),
        environment,
    );
    session
        .open(
            test_support::lsp::uri_of(&dir.join("main.jai")),
            1,
            "main :: () {}".into(),
        )
        .unwrap();
    assert!(!session.warm_auto_import_unless_interrupted());
    assert!(
        fs.reads
            .borrow()
            .iter()
            .any(|p| p.ends_with("A/module.jai"))
    );
    reset();
    fs.reads.borrow_mut().clear();
    assert!(session.warm_auto_import_unless_interrupted());
    let reads = fs.reads.borrow();
    assert!(
        !reads.iter().any(|p| p.ends_with("A/module.jai")),
        "completed work was discarded: {reads:?}"
    );
    assert!(
        reads.iter().any(|p| p.ends_with("B/module.jai")),
        "the interrupted file must be retried"
    );
}
