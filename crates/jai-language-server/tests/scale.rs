//! Large inputs and compile-time code that cannot finish: the server answers instead of stalling,
//! dropping the document or dying.
use jai_language_server::{DocumentUri, Environment, Limits, Session};
use std::path::PathBuf;
use std::time::{Duration, Instant};

fn environment() -> Environment {
    let stdlib = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../stdlib");
    Environment {
        fs: std::rc::Rc::new(jaic::sema::NativeFs),
        options: Box::new(move |_| {
            let mut options = jaic::sema::Options::host();
            options.import_paths = vec![stdlib.clone()];
            options.preload = Some(stdlib.join("prelude/module.jai"));
            options
        }),
    }
}

/// Focus's `src/main.jai` imports `Basic` with a parameter its build metaprogram defines, so opened
/// alone the import never resolves and dozens of language files `#insert` code that waits for it.
/// Settling those items used to expand them again for every compile-time run beneath them, which
/// grew exponentially: no diagnostics within ten minutes, several GiB. Skipped without the corpus.
#[test]
fn project_with_an_unresolvable_import_still_answers() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../corpus/upstream/focus-editor--focus/src/main.jai");
    let Ok(text) = std::fs::read_to_string(&root) else {
        return;
    };
    let mut s = Session::with_environment(Limits::default(), environment());
    let uri = DocumentUri::parse(&format!(
        "file://{}",
        root.canonicalize().unwrap().display()
    ))
    .unwrap();
    let started = Instant::now();
    s.open(uri.clone(), 1, text).unwrap();
    let diagnostics = s.diagnostics(&uri).unwrap();
    assert!(!diagnostics.is_empty());
    assert!(
        started.elapsed() < Duration::from_secs(60),
        "took {:?}",
        started.elapsed()
    );
}
