//! URI projection must not reinterpret accepted compiler locations using later alias reads.
#![cfg(unix)]
use super::projection_api as api;
use jaic::sema::{FileSystem, NativeFs};
use std::{
    cell::Cell,
    path::{Path, PathBuf},
    rc::Rc,
};
use test_support::lsp::at_nth;

pub(crate) struct RetargetFs {
    pub(crate) alias: PathBuf,
    pub(crate) second: PathBuf,
    pub(crate) canonical_calls: Cell<usize>,
    pub(crate) retarget_at: Cell<Option<usize>>,
    pub(crate) changed: Cell<bool>,
    pub(crate) retarget_after: bool,
}

impl FileSystem for RetargetFs {
    fn is_native(&self) -> bool {
        true
    }

    fn read(&self, path: &Path) -> Option<Vec<u8>> {
        NativeFs.read(path)
    }

    fn is_file(&self, path: &Path) -> bool {
        NativeFs.is_file(path)
    }

    fn is_dir(&self, path: &Path) -> bool {
        NativeFs.is_dir(path)
    }

    fn list_dir(&self, path: &Path) -> Vec<(String, bool)> {
        NativeFs.list_dir(path)
    }

    fn stamp(&self, path: &Path) -> Option<u64> {
        NativeFs.stamp(path)
    }

    fn canonical_exact(&self, path: &Path) -> Option<PathBuf> {
        NativeFs.canonical_exact(path)
    }

    fn canonical(&self, path: &Path) -> PathBuf {
        let before = self.retarget_after.then(|| NativeFs.canonical(path));
        if path == self.alias.join("main.jai") {
            let call = self.canonical_calls.get() + 1;
            self.canonical_calls.set(call);
            if self.retarget_at.get() == Some(call) {
                self.retarget_at.set(None);
                std::fs::remove_file(&self.alias).unwrap();
                std::os::unix::fs::symlink(&self.second, &self.alias).unwrap();
                self.changed.set(true);
            }
        }
        before.unwrap_or_else(|| NativeFs.canonical(path))
    }
}

pub(crate) const REFERENCES: &str =
    "consume :: (value: int) {}\nmain :: () { local := 1; local += 2; consume(local); }\n";

/// Share actual native retargeting across public API and private eviction reducers.
pub(crate) fn exercise_query(
    before_warm: impl FnOnce(&mut api::Session),
    retained: bool,
    source: &str,
    occurrence: usize,
    expected_count: usize,
    request: impl Fn(
        &api::Session,
        &api::DocumentUri,
        api::Position,
    ) -> Result<Vec<api::Location>, api::Error>,
) {
    exercise_query_with_settled_setup(
        before_warm,
        |_| {},
        retained,
        source,
        occurrence,
        expected_count,
        request,
    );
}

/// Seed synthetic producer facts outside the armed consumer transaction.
/// For example, labelled lint fixtures seed the initial and later stable compiler owners.
pub(crate) fn exercise_query_with_settled_setup(
    before_warm: impl FnOnce(&mut api::Session),
    before_settled: impl FnOnce(&mut api::Session),
    retained: bool,
    source: &str,
    occurrence: usize,
    expected_count: usize,
    request: impl Fn(
        &api::Session,
        &api::DocumentUri,
        api::Position,
    ) -> Result<Vec<api::Location>, api::Error>,
) {
    let temp = test_support::TempDir::new("postguard-uri-projection");
    let events = test_support::TempDir::new("projection-capture-events");
    let event = events.path().join("changed-input");
    std::fs::write(&event, "known native event").unwrap();
    let first = temp.path().join("first");
    let second = temp.path().join("second");
    let alias = temp.path().join("alias");
    for dir in [&first, &second] {
        std::fs::create_dir(dir).unwrap();
        std::fs::write(dir.join("main.jai"), source).unwrap();
    }
    std::os::unix::fs::symlink(&first, &alias).unwrap();
    let fs = Rc::new(RetargetFs {
        alias: alias.clone(),
        second,
        canonical_calls: Cell::new(0),
        retarget_at: Cell::new(None),
        changed: Cell::new(false),
        retarget_after: false,
    });
    let mut session = api::Session::with_environment(
        api::Limits::default(),
        api::Environment {
            fs: fs.clone(),
            services: None,
            options: Box::new(|_, _| {
                let mut options = jaic::sema::Options::host();
                options.runtime_support = false;
                options.stack_trace = false;
                options
            }),
        },
    );
    let uri = api::DocumentUri::from_path(&alias.join("main.jai")).unwrap();
    let position = at_nth(source, "local", occurrence, 1);
    let query = |session: &api::Session| request(session, &uri, position);
    session.open(uri.clone(), 1, source.into()).unwrap();
    session.diagnostics(&uri).unwrap();
    before_warm(&mut session);
    let initial = query(&session).unwrap();
    assert_eq!(initial.len(), expected_count);
    let compiled = session.compile_counts().0;
    // Each native retarget measurement explicitly begins a new accepted-input
    // transaction. Unchanged warm epochs are covered separately and do no native IO.
    session.file_changed(&event, false);
    fs.canonical_calls.set(0);
    let expected = query(&session).unwrap();
    assert_eq!(expected.len(), expected_count);
    assert!(expected.iter().all(|at| at.uri == uri.as_str()));
    let warm_calls = fs.canonical_calls.get();
    session.file_changed(&event, false);
    fs.canonical_calls.set(0);
    assert_eq!(query(&session).unwrap(), expected);
    assert_eq!(
        fs.canonical_calls.get(),
        warm_calls,
        "stable unchanged-query setup"
    );
    // Arm real alias churn from the same stable native admission trace.
    // Epoch replay adds canonical calls, so this ordinal does not identify output mapping alone.
    assert!(warm_calls >= expected.len());
    session.file_changed(&event, false);
    fs.retarget_at.set(Some(warm_calls - expected.len() + 1));
    fs.canonical_calls.set(0);
    let found = match query(&session) {
        Ok(found) => found,
        Err(api::Error::DiagnosticsUnavailable) => Vec::new(),
        Err(error) => panic!("unexpected projection error: {error:?}"),
    };
    assert!(
        fs.changed.get(),
        "fixture must retarget the real alias during mapping"
    );
    assert!(
        found.is_empty() || found == expected,
        "mixed accepted facts with later URI identity: {found:?}"
    );
    assert_eq!(
        session.compile_counts().0,
        compiled,
        "unstable projection must not retry a compile"
    );
    // The alias now names another semantic root. Retained old-root facts cannot authorize
    // that new context; an explicit later request may perform its fresh check.
    if retained {
        session.defer_compiles(false);
    }
    before_settled(&mut session);
    let settled = query(&session).unwrap();
    assert_eq!(
        settled, expected,
        "a later stable request restores useful references"
    );
    assert_eq!(
        session.compile_counts().0,
        compiled + 1,
        "new root needs exactly one new check"
    );
}
