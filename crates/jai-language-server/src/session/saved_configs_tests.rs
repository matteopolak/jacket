use super::*;
use crate::{Environment, Limits};
use std::cell::{Cell, RefCell};
use std::rc::Rc;

type AfterRead = Box<dyn FnOnce()>;

pub(super) struct ControlledFs {
    config: PathBuf,
    native: bool,
    fail: Cell<bool>,
    reads: Cell<usize>,
    ops: Cell<usize>,
    missing_calls: Cell<usize>,
    after_missing: RefCell<Option<(usize, AfterRead)>>,
    other_failure: RefCell<Option<(PathBuf, bool)>>,
    after_read: RefCell<Option<AfterRead>>,
}

impl FileSystem for ControlledFs {
    fn read(&self, path: &Path) -> Option<Vec<u8>> {
        self.ops.set(self.ops.get() + 1);
        if self
            .other_failure
            .borrow()
            .as_ref()
            .is_some_and(|(other, failed)| other == path && *failed)
        {
            return None;
        }
        if path != self.config {
            return NativeFs.read(path);
        }
        self.reads.set(self.reads.get() + 1);
        if self.fail.get() {
            return None;
        }
        let bytes = NativeFs.read(path);
        if let Some(after) = self.after_read.borrow_mut().take() {
            after();
        }
        bytes
    }

    fn is_file(&self, path: &Path) -> bool {
        self.ops.set(self.ops.get() + 1);
        NativeFs.is_file(path)
    }

    fn is_dir(&self, path: &Path) -> bool {
        self.ops.set(self.ops.get() + 1);
        NativeFs.is_dir(path)
    }

    fn canonical(&self, path: &Path) -> PathBuf {
        self.ops.set(self.ops.get() + 1);
        NativeFs.canonical(path)
    }

    fn canonical_exact(&self, path: &Path) -> Option<PathBuf> {
        self.ops.set(self.ops.get() + 1);
        NativeFs.canonical_exact(path)
    }

    fn list_dir(&self, path: &Path) -> Vec<(String, bool)> {
        self.ops.set(self.ops.get() + 1);
        NativeFs.list_dir(path)
    }

    fn missing_file_key(&self, path: &Path) -> Option<PathBuf> {
        self.ops.set(self.ops.get() + 1);
        let value = NativeFs.missing_file_key(path);
        if path == self.config {
            self.missing_calls.set(self.missing_calls.get() + 1);
            let fire = self
                .after_missing
                .borrow()
                .as_ref()
                .is_some_and(|(at, _)| *at == self.missing_calls.get());
            if fire && let Some((_, action)) = self.after_missing.borrow_mut().take() {
                action();
            }
        }
        value
    }

    fn stamp(&self, path: &Path) -> Option<u64> {
        self.ops.set(self.ops.get() + 1);
        NativeFs.stamp(path)
    }

    fn is_native(&self) -> bool {
        self.native
    }
}

fn session(fs: Rc<dyn FileSystem>) -> Session {
    Session::with_environment(
        Limits::default(),
        Environment {
            fs,
            services: None,
            options: Box::new(|_, _| {
                let mut options = jaic::sema::Options::host();
                options.runtime_support = false;
                options.stack_trace = false;
                options
            }),
        },
    )
}

#[test]
fn unavailable_settings_reject_publication_without_a_native_epoch() {
    let temp = test_support::TempDir::new("saved-config-untracked-failure");
    let config = temp.path().join("jacket.toml");
    let main = temp.path().join("main.jai");
    let text = "[project]\nentry=[\"main.jai\"]\n";
    std::fs::write(&config, text).unwrap();
    std::fs::write(&main, "main :: () {}").unwrap();
    let fs = Rc::new(ControlledFs {
        config: config.clone(),
        native: false,
        fail: Cell::new(false),
        reads: Cell::new(0),
        ops: Cell::new(0),
        missing_calls: Cell::new(0),
        after_missing: RefCell::default(),
        other_failure: RefCell::default(),
        after_read: RefCell::default(),
    });
    let mut session = session(fs.clone());
    let config_uri = DocumentUri::from_path(&config).unwrap();
    let main_uri = DocumentUri::from_path(&main).unwrap();
    session.open(config_uri.clone(), 1, text.into()).unwrap();
    session
        .open(main_uri.clone(), 1, "main :: () {}".into())
        .unwrap();
    assert!(session.input_epoch.is_none());
    assert!(session.diagnostics(&main_uri).unwrap().is_empty());
    fs.fail.set(true);
    session.saved_file_changed(&config, false);
    assert!(session.saved_configs[&config_uri].unavailable());
    assert_eq!(
        session.diagnostics(&main_uri),
        Err(Error::DiagnosticsUnavailable),
        "config defaults cannot turn a known failed refresh into a clean report"
    );
    fs.fail.set(false);
    session.saved_file_changed(&config, false);
    assert_eq!(session.saved_configs[&config_uri].text(), Some(text));
    assert!(session.diagnostics(&main_uri).unwrap().is_empty());
}

#[cfg(unix)]
#[test]
fn retarget_during_saved_capture_rejects_mixed_identity_and_text() {
    let temp = test_support::TempDir::new("saved-config-retarget-capture");
    let first = temp.path().join("first.toml");
    let second = temp.path().join("second.toml");
    let config = temp.path().join("jacket.toml");
    let original = "[project]\nentry=[]\n";
    let changed = "[project]\nentry=[]\n# new target\n";
    std::fs::write(&first, original).unwrap();
    std::fs::write(&second, changed).unwrap();
    std::os::unix::fs::symlink(&first, &config).unwrap();
    let fs = Rc::new(ControlledFs {
        config: config.clone(),
        native: true,
        fail: Cell::new(false),
        reads: Cell::new(0),
        ops: Cell::new(0),
        missing_calls: Cell::new(0),
        after_missing: RefCell::default(),
        other_failure: RefCell::default(),
        after_read: RefCell::default(),
    });
    let mut session = session(fs.clone());
    let uri = DocumentUri::from_path(&config).unwrap();
    session.open(uri.clone(), 1, original.into()).unwrap();
    let old_identity = session.saved_configs[&uri].identity.clone();
    fs.reads.set(0);
    let alias = config.clone();
    *fs.after_read.borrow_mut() = Some(Box::new(move || {
        std::fs::remove_file(&alias).unwrap();
        std::os::unix::fs::symlink(second, alias).unwrap();
    }));
    let compiles = session.compile_counts().0;
    session.saved_file_changed(&config, false);
    let rejected = &session.saved_configs[&uri];
    assert!(rejected.unavailable());
    assert!(rejected.text().is_none());
    assert_eq!(rejected.identity, old_identity);
    assert_eq!(session.compile_counts().0, compiles);
    assert_eq!(
        fs.reads.get(),
        2,
        "one capture and failed replay, no same-event retry"
    );
    session.saved_file_changed(&config, false);
    assert_eq!(session.saved_configs[&uri].text(), Some(changed));
    assert_ne!(session.saved_configs[&uri].identity, old_identity);
    assert_eq!(session.lint_configs[&uri].text, original);
}

#[cfg(unix)]
#[test]
fn a_physical_event_refreshes_every_saved_alias_binding() {
    let temp = test_support::TempDir::new("saved-config-multiple-aliases");
    let physical = temp.path().join("physical.toml");
    let original = "[project]\nentry=[]\n";
    let changed = "[project]\nentry=[]\n# updated\n";
    std::fs::write(&physical, original).unwrap();
    let mut session = session(Rc::new(NativeFs));
    let mut uris = Vec::new();
    for name in ["first", "second"] {
        let directory = temp.path().join(name);
        std::fs::create_dir(&directory).unwrap();
        let alias = directory.join("jacket.toml");
        let initial_target = if name == "second" {
            let separate = temp.path().join("separate.toml");
            std::fs::write(&separate, original).unwrap();
            separate
        } else {
            physical.clone()
        };
        std::os::unix::fs::symlink(initial_target, &alias).unwrap();
        let uri = DocumentUri::from_path(&alias).unwrap();
        session.open(uri.clone(), 1, original.into()).unwrap();
        uris.push(uri);
    }
    // Public open rejects aliases already bound to one file. Two stored spellings can
    // converge later through a missed external retarget, before this known event.
    let second_alias = temp.path().join("second/jacket.toml");
    std::fs::remove_file(&second_alias).unwrap();
    std::os::unix::fs::symlink(&physical, second_alias).unwrap();
    std::fs::write(&physical, changed).unwrap();
    session.saved_file_changed(&physical, false);
    for uri in &uris {
        assert_eq!(session.saved_configs[uri].text(), Some(changed));
        assert_eq!(session.lint_configs[uri].text, original);
    }
    let first_alias = temp.path().join("first/jacket.toml");
    let replacement = temp.path().join("replacement.toml");
    let latest = "[project]\nentry=[]\n# independent replacement\n";
    std::fs::write(&replacement, latest).unwrap();
    std::fs::remove_file(&first_alias).unwrap();
    std::os::unix::fs::symlink(replacement, &first_alias).unwrap();
    session.saved_file_changed(&first_alias, false);
    assert_eq!(session.saved_configs[&uris[0]].text(), Some(latest));
    assert_eq!(session.saved_configs[&uris[1]].text(), Some(changed));
    assert!(session.saved_config_for_use(&uris[0]).is_some());
    assert!(
        session.saved_config_for_use(&uris[1]).is_some(),
        "the unchanged owner must not inherit the other alias's old capture prefix"
    );
    assert!(session.accept_input_epoch());
}

#[test]
fn unavailable_native_settings_recover_between_requests_without_a_new_client_event() {
    let temp = test_support::TempDir::new("saved-config-native-recovery");
    let config = temp.path().join("jacket.toml");
    let main = temp.path().join("main.jai");
    let text = "[project]\nentry=[\"main.jai\"]\n";
    std::fs::write(&config, text).unwrap();
    std::fs::write(&main, "main :: () {}").unwrap();
    let fs = Rc::new(ControlledFs {
        config: config.clone(),
        native: true,
        fail: Cell::new(false),
        reads: Cell::new(0),
        ops: Cell::new(0),
        missing_calls: Cell::new(0),
        after_missing: RefCell::default(),
        other_failure: RefCell::default(),
        after_read: RefCell::default(),
    });
    let mut session = session(fs.clone());
    let config_uri = DocumentUri::from_path(&config).unwrap();
    let main_uri = DocumentUri::from_path(&main).unwrap();
    session.open(config_uri.clone(), 1, text.into()).unwrap();
    session
        .open(main_uri.clone(), 1, "main :: () {}".into())
        .unwrap();
    assert!(session.diagnostics(&main_uri).unwrap().is_empty());
    fs.fail.set(true);
    session.saved_file_changed(&config, false);
    assert!(session.saved_configs[&config_uri].unavailable());
    assert_eq!(
        session.diagnostics(&main_uri),
        Err(Error::DiagnosticsUnavailable)
    );
    fs.fail.set(false);
    let compiles = session.compile_counts().0;
    let mut recovered = false;
    for step in 0..256 {
        // Typing changes overlay generation; new unrelated capture keys change the map,
        // but neither is a known native-event replacement of the candidate's authority.
        let edited = format!("main :: () {{}} // {step}");
        session
            .change(
                &main_uri,
                step + 2,
                &[crate::TextChange {
                    range: None,
                    range_length: None,
                    text: edited.clone(),
                }],
            )
            .unwrap();
        assert_eq!(session.document_text(&main_uri).unwrap(), edited);
        let unrelated = temp.path().join(format!("unrelated-{step}.txt"));
        let _ = session.environment.as_ref().unwrap().fs.read(&unrelated);
        let before = fs.ops.get();
        session.reconcile_input_epoch();
        assert!(
            fs.ops.get() - before <= 1,
            "capture/replay/commit share one native-operation allowance"
        );
        if step == 0 {
            assert_eq!(
                fs.ops.get(),
                before,
                "candidate setup performs no native IO"
            );
        }
        if session.saved_configs[&config_uri].text() == Some(text) {
            recovered = true;
            break;
        }
    }
    assert!(
        recovered,
        "reconciliation must observe recovered native input without another client event"
    );
    assert_eq!(
        session.compile_counts().0,
        compiles,
        "filesystem recovery does not compile inline"
    );
    assert!(session.diagnostics(&main_uri).unwrap().is_empty());
}

pub(super) fn unavailable_native_config(
    tag: &str,
) -> (
    test_support::TempDir,
    Rc<ControlledFs>,
    Session,
    DocumentUri,
) {
    let temp = test_support::TempDir::new(tag);
    let config = temp.path().join("jacket.toml");
    let text = "[project]\nentry=[]\n";
    std::fs::write(&config, text).unwrap();
    let fs = Rc::new(ControlledFs {
        config: config.clone(),
        native: true,
        fail: Cell::new(false),
        reads: Cell::new(0),
        ops: Cell::new(0),
        missing_calls: Cell::new(0),
        after_missing: RefCell::default(),
        other_failure: RefCell::default(),
        after_read: RefCell::default(),
    });
    let mut session = session(fs.clone());
    let uri = DocumentUri::from_path(&config).unwrap();
    session.open(uri.clone(), 1, text.into()).unwrap();
    fs.fail.set(true);
    session.saved_file_changed(&config, false);
    assert!(session.saved_configs[&uri].unavailable());
    fs.fail.set(false);
    fs.reads.set(0);
    fs.missing_calls.set(0);
    (temp, fs, session, uri)
}

fn reject_final_recovery_replay(cancel: bool) {
    let (_temp, fs, mut session, uri) = unavailable_native_config("saved-config-final-replay");
    let fired = Rc::new(Cell::new(false));
    let observed = fired.clone();
    let parent = Rc::downgrade(session.input_epoch.as_ref().unwrap());
    *fs.after_missing.borrow_mut() = Some((
        2,
        Box::new(move || {
            if cancel {
                jaic::interrupt::request(true);
                assert!(jaic::interrupt::should_stop());
                assert!(jaic::interrupt::fired());
            } else {
                parent.upgrade().unwrap().begin_capture();
            }
            observed.set(true);
        }),
    ));
    for _ in 0..128 {
        let before = fs.ops.get();
        session.reconcile_input_epoch();
        assert!(fs.ops.get() - before <= 1);
        if fired.get() {
            break;
        }
    }
    assert!(
        fired.get(),
        "the final actual replay boundary was exercised"
    );
    assert!(session.saved_configs[&uri].unavailable());
    assert!(session.saved_configs[&uri].text().is_none());
    assert!(session.config_recovery.borrow().is_none());
    assert_eq!(session.compile_counts().0, 0);
    // The owner has consumed the interrupting input before the later stable transaction.
    if cancel {
        jaic::interrupt::request(false);
    }
    for _ in 0..128 {
        session.reconcile_input_epoch();
        if session.saved_configs[&uri].text().is_some() {
            break;
        }
    }
    assert!(
        session.saved_configs[&uri].text().is_some(),
        "a later stable transaction recovers"
    );
}

#[test]
fn native_event_during_final_recovery_replay_rejects_the_candidate() {
    reject_final_recovery_replay(false);
}

#[test]
fn cancellation_during_final_recovery_replay_rejects_the_candidate() {
    reject_final_recovery_replay(true);
}

#[test]
fn recovery_does_not_starve_other_unavailable_settings() {
    let (temp, fs, mut session, first) = unavailable_native_config("saved-config-fair-recovery");
    let second = temp.path().join("other/jacket.toml");
    std::fs::create_dir_all(second.parent().unwrap()).unwrap();
    let text = "[project]\nentry=[]\n";
    std::fs::write(&second, text).unwrap();
    let second_uri = DocumentUri::from_path(&second).unwrap();
    session.open(second_uri.clone(), 1, text.into()).unwrap();
    fs.fail.set(true);
    *fs.other_failure.borrow_mut() = Some((second.clone(), true));
    session.saved_file_changed(&fs.config, false);
    session.saved_file_changed(&second, false);
    *fs.other_failure.borrow_mut() = Some((second, false));
    fs.reads.set(0);
    for _ in 0..256 {
        let before = fs.ops.get();
        session.reconcile_input_epoch();
        assert!(fs.ops.get() - before <= 1);
        if session.saved_configs[&second_uri].text().is_some() {
            break;
        }
    }
    assert_eq!(session.saved_configs[&second_uri].text(), Some(text));
    assert!(session.saved_configs[&first].unavailable());
    assert_eq!(
        fs.reads.get(),
        2,
        "one failed capture/replay, then a different owner gets service"
    );
    assert_eq!(session.compile_counts().0, 0);
    fs.fail.set(false);
    for _ in 0..256 {
        session
            .service_config_recovery(std::time::Instant::now() + std::time::Duration::from_secs(1));
        if session.saved_configs[&first].text().is_some() {
            break;
        }
    }
    assert_eq!(session.saved_configs[&first].text(), Some(text));
}

#[test]
fn budget_blocked_recovery_waits_for_capacity_and_reserves_pending_bytes() {
    let (temp, fs, mut session, uri) = unavailable_native_config("saved-config-capacity-recovery");
    session.limits.workspace_bytes = 64 * 1024;
    let other = DocumentUri::from_path(&temp.path().join("other.jai")).unwrap();
    session
        .open(other.clone(), 1, format!("//{}", "x".repeat(40 * 1024)))
        .unwrap();
    let grown = format!("[project]\nentry=[]\n#{}", "x".repeat(12 * 1024));
    std::fs::write(&fs.config, &grown).unwrap();
    session.saved_file_changed(&fs.config, false);
    assert!(matches!(
        session.saved_configs[&uri].state,
        SavedConfigState::Unavailable {
            reason: ConfigFailure::Budget { .. },
            ..
        }
    ));
    assert_eq!(
        session
            .input_epoch
            .as_ref()
            .unwrap()
            .captured_read_bytes(&fs.config),
        0,
        "a workspace-rejected event snapshot cannot publish provisional bytes into the parent epoch"
    );
    fs.reads.set(0);
    for _ in 0..16 {
        session.reconcile_input_epoch();
    }
    assert_eq!(
        fs.reads.get(),
        0,
        "unchanged insufficient capacity cannot cause repeated file reads"
    );
    session.close(&other).unwrap();
    let mut checked_reservation = false;
    for _ in 0..128 {
        let before = fs.ops.get();
        session.reconcile_input_epoch();
        assert!(fs.ops.get() - before <= 1);
        let holds_payload = session
            .config_recovery
            .borrow()
            .as_ref()
            .is_some_and(|candidate| candidate.storage() > grown.len());
        if holds_payload && !checked_reservation {
            let extra = DocumentUri::from_path(&temp.path().join("extra.jai")).unwrap();
            assert_eq!(
                session.open(extra, 1, format!("//{}", "x".repeat(40 * 1024))),
                Err(Error::Limit("workspace byte budget exceeded"))
            );
            assert!(
                session.config_recovery.borrow().is_some(),
                "rejected admission preserves the candidate"
            );
            checked_reservation = true;
        }
        if session.saved_configs[&uri].text().is_some() {
            break;
        }
    }
    assert!(checked_reservation);
    assert_eq!(session.saved_configs[&uri].text(), Some(grown.as_str()));
    assert_eq!(session.compile_counts().0, 0);
}

#[test]
fn oversized_recovery_keeps_the_exact_accounted_read_owner() {
    let (_temp, fs, mut session, uri) = unavailable_native_config("saved-config-large-recovery");
    let grown = format!("[project]\nentry=[]\n#{}", "x".repeat(17 * 1024 * 1024));
    std::fs::write(&fs.config, &grown).unwrap();
    for _ in 0..128 {
        let before = fs.ops.get();
        session.reconcile_input_epoch();
        assert!(fs.ops.get() - before <= 1);
        if session.saved_configs[&uri].text().is_some() {
            break;
        }
    }
    assert_eq!(session.saved_configs[&uri].text(), Some(grown.as_str()));
    assert!(
        session.saved_configs[&uri].storage(&uri) > grown.len() * 2,
        "the saved text, byte proof and metadata are charged"
    );
    assert!(
        session.input_epoch.as_ref().unwrap().retained_bytes() < 1024 * 1024,
        "the native carrier borrows the proof owner instead of retaining another large payload"
    );
    assert_eq!(
        fs.reads.get(),
        2,
        "one capture and one replay; commit is filesystem-pure"
    );
    assert_eq!(session.compile_counts().0, 0);
}

#[test]
fn pending_recovery_budget_includes_the_coexisting_unavailable_owner() {
    let (_temp, fs, mut session, uri) = unavailable_native_config("saved-config-coexisting-budget");
    session.reconcile_input_epoch();
    let candidate = session.config_recovery.borrow_mut().take().unwrap();
    let pending = candidate.storage();
    drop(candidate);
    let owner = session.saved_configs[&uri].storage(&uri);
    let buffer = session.lint_configs[&uri].text.len() + uri.as_str().len();
    assert!(owner > 1);
    session.limits.workspace_bytes = buffer + owner + pending - 1;
    let before = fs.ops.get();
    session.reconcile_input_epoch();
    assert_eq!(fs.ops.get(), before, "admission/setup does no native IO");
    assert!(
        session.config_recovery.borrow().is_none(),
        "the old unavailable owner is still live while a recovery candidate is pending"
    );
    assert!(matches!(
        session.saved_configs[&uri].state,
        SavedConfigState::Unavailable {
            reason: ConfigFailure::Budget { .. },
            ..
        }
    ));
}
