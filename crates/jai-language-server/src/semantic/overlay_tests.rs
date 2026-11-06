//! Overlay directory enumeration obeys the same visible filesystem as source reads.
use super::*;
use jaic::sema::VirtualFs;

fn overlay() -> OverlayFs {
    let mut base = VirtualFs::default();
    base.insert("/project/saved.jai", "disk".as_bytes());
    base.insert("/project/nested/on-disk.jai", "disk".as_bytes());
    OverlayFs {
        base: Rc::new(base),
        dir: PathBuf::from("/project"),
        files: RefCell::new(BTreeMap::from([
            (
                PathBuf::from("/project/saved.jai"),
                Arc::from(&b"unsaved change"[..]),
            ),
            (
                PathBuf::from("/project/new.jai"),
                Arc::from(&b"unsaved file"[..]),
            ),
            (
                PathBuf::from("/project/generated/child.jai"),
                Arc::from(&b"nested"[..]),
            ),
        ])),
        generated: BTreeMap::new(),
        canonical: jaic::paths::CanonCache::default(),
        reads: Rc::default(),
    }
}

fn listing(fs: &OverlayFs, path: &str) -> Vec<(String, bool)> {
    let mut found = fs.list_dir(Path::new(path));
    found.sort();
    found
}

#[test]
fn listings_include_unsaved_documents_and_inferred_directories_without_duplicates() {
    let fs = overlay();
    assert_eq!(
        listing(&fs, "."),
        [
            ("generated".into(), true),
            ("nested".into(), true),
            ("new.jai".into(), false),
            ("saved.jai".into(), false)
        ]
    );
    assert_eq!(
        listing(&fs, "/project/generated"),
        [("child.jai".into(), false)]
    );
    assert_eq!(fs.read(Path::new("saved.jai")).unwrap(), b"unsaved change");
    assert!(
        fs.reads
            .borrow()
            .concerns(Path::new("/project/generated/later.jai"))
    );
}

#[test]
fn closing_and_renaming_overlays_change_listings_without_hiding_disk_files() {
    let mut fs = overlay();
    let text = fs
        .files
        .get_mut()
        .remove(Path::new("/project/new.jai"))
        .unwrap();
    fs.files
        .get_mut()
        .insert(PathBuf::from("/project/renamed.jai"), text);
    fs.files.get_mut().remove(Path::new("/project/saved.jai"));
    let rows = listing(&fs, "/project");
    assert!(!rows.iter().any(|(name, _)| name == "new.jai"));
    assert!(rows.contains(&("renamed.jai".into(), false)));
    assert!(
        rows.contains(&("saved.jai".into(), false)),
        "closing restores saved disk file"
    );
    assert_eq!(fs.read(Path::new("saved.jai")).unwrap(), b"disk");
    fs.files
        .get_mut()
        .remove(Path::new("/project/generated/child.jai"));
    assert!(
        !listing(&fs, "/project")
            .iter()
            .any(|(name, _)| name == "generated")
    );
}

#[test]
fn directory_stamps_change_with_overlay_names_and_record_directory_dependencies() {
    let mut fs = overlay();
    let before = fs.stamp(Path::new("/project")).expect("directory stamp");
    fs.files.get_mut().remove(Path::new("/project/new.jai"));
    let after = fs
        .stamp(Path::new("/project"))
        .expect("directory stamp after close");
    assert_ne!(before, after);
    assert!(fs.reads.borrow().concerns(Path::new("/project/later.jai")));
}

#[cfg(unix)]
#[test]
fn direct_reads_remember_directory_aliases_even_without_a_loader_canonical_query() {
    let dir = test_support::TempDir::new("overlay-directory-alias");
    let root = std::fs::canonicalize(dir.path()).unwrap();
    std::fs::create_dir(root.join("target")).unwrap();
    std::fs::write(root.join("target/data.txt"), "one").unwrap();
    let alias = root.join("alias");
    std::os::unix::fs::symlink(root.join("target"), &alias).unwrap();
    let fs = OverlayFs {
        base: Rc::new(jaic::sema::NativeFs),
        dir: root,
        files: RefCell::new(BTreeMap::new()),
        generated: BTreeMap::new(),
        canonical: jaic::paths::CanonCache::default(),
        reads: Rc::default(),
    };
    assert_eq!(fs.read(&alias.join("data.txt")).unwrap(), b"one");
    std::fs::remove_file(&alias).unwrap();
    assert!(
        fs.reads.borrow().concerns_tree(&alias),
        "removing the observed directory alias invalidates its read"
    );
}

#[cfg(unix)]
#[test]
fn a_retargeted_unread_alias_records_its_new_dependency_after_path_cache_clear() {
    let dir = test_support::TempDir::new("overlay-retarget-unread");
    let root = std::fs::canonicalize(dir.path()).unwrap();
    let first = root.join("first.txt");
    let second = root.join("second.txt");
    let alias = root.join("alias.txt");
    std::fs::write(&first, "one").unwrap();
    std::fs::write(&second, "two").unwrap();
    std::os::unix::fs::symlink(&first, &alias).unwrap();
    let fs = OverlayFs {
        base: Rc::new(jaic::sema::NativeFs),
        dir: root,
        files: RefCell::new(BTreeMap::new()),
        generated: BTreeMap::new(),
        canonical: jaic::paths::CanonCache::default(),
        reads: Rc::default(),
    };
    assert_eq!(fs.canonical(&alias), first);
    assert!(
        !fs.reads.borrow().concerns(&alias),
        "identity-only probe is not a content dependency"
    );
    std::fs::remove_file(&alias).unwrap();
    std::os::unix::fs::symlink(&second, &alias).unwrap();
    fs.canonical.clear();
    assert_eq!(fs.canonical(&alias), second);
    assert_eq!(fs.read(&second).unwrap(), b"two");
    assert!(
        fs.reads.borrow().concerns(&alias),
        "the new observed target is now a dependency"
    );
}

#[test]
fn replacing_open_buffers_preserves_the_compilers_generated_sources() {
    let mut fs = overlay();
    let generated = PathBuf::from("/project/generated-root.jai");
    fs.generated
        .insert(generated.clone(), Arc::from(&b"#import \"Library\";"[..]));
    fs.replace_files(BTreeMap::from([
        (
            generated.clone(),
            Arc::from(&b"client spelling must not replace synthetic root"[..]),
        ),
        (
            PathBuf::from("/project/current.jai"),
            Arc::from(&b"current"[..]),
        ),
    ]));
    assert_eq!(fs.read(&generated).unwrap(), b"#import \"Library\";");
    assert_eq!(
        fs.read(Path::new("/project/current.jai")).unwrap(),
        b"current"
    );
    assert!(fs.read(Path::new("/project/new.jai")).is_none());
    assert!(fs.is_file(&generated));
}

#[test]
fn silent_native_directory_changes_invalidate_the_exact_listing_proof() {
    let temp = test_support::TempDir::new("native-listing-proof");
    let root = temp.path().join("private");
    std::fs::create_dir(&root).unwrap();
    let fs = OverlayFs {
        base: Rc::new(jaic::sema::NativeFs),
        dir: root.clone(),
        files: RefCell::default(),
        generated: BTreeMap::new(),
        canonical: jaic::paths::CanonCache::default(),
        reads: Rc::default(),
    };
    assert!(fs.list_dir(&root).is_empty());
    assert!(fs.reads.borrow().is_current(&*fs.base));
    std::fs::write(root.join("new.txt"), "new").unwrap();
    assert!(!fs.reads.borrow().is_current(&*fs.base));
    *fs.reads.borrow_mut() = Reads::default();
    assert_eq!(fs.list_dir(&root), [("new.txt".into(), false)]);
    std::fs::remove_file(root.join("new.txt")).unwrap();
    assert!(!fs.reads.borrow().is_current(&*fs.base));
}

#[test]
fn conflicting_native_reads_never_replace_the_first_observation() {
    let temp = test_support::TempDir::new("native-mixed-proof");
    let root = temp.path().to_path_buf();
    let path = root.join("data.txt");
    std::fs::write(&path, "first").unwrap();
    let fs = OverlayFs {
        base: Rc::new(jaic::sema::NativeFs),
        dir: root,
        files: RefCell::default(),
        generated: BTreeMap::new(),
        canonical: jaic::paths::CanonCache::default(),
        reads: Rc::default(),
    };
    assert_eq!(fs.read(&path).unwrap(), b"first");
    std::fs::write(&path, "other").unwrap();
    assert_eq!(fs.read(&path).unwrap(), b"other");
    std::fs::write(&path, "first").unwrap();
    assert!(
        !fs.reads.borrow().is_current(&*fs.base),
        "restoring bytes cannot accept a mixed context"
    );
}

struct MissingProbeFs {
    canonical_calls: std::cell::Cell<usize>,
    enabled: std::cell::Cell<bool>,
    create_at: RefCell<Option<PathBuf>>,
    stop_at: RefCell<Option<PathBuf>>,
}

impl MissingProbeFs {
    fn new() -> Self {
        Self {
            canonical_calls: std::cell::Cell::new(0),
            enabled: std::cell::Cell::new(true),
            create_at: RefCell::default(),
            stop_at: RefCell::default(),
        }
    }
}

impl FileSystem for MissingProbeFs {
    fn is_native(&self) -> bool {
        true
    }

    fn read(&self, path: &Path) -> Option<Vec<u8>> {
        jaic::sema::NativeFs.read(path)
    }

    fn is_file(&self, path: &Path) -> bool {
        jaic::sema::NativeFs.is_file(path)
    }

    fn is_dir(&self, path: &Path) -> bool {
        jaic::sema::NativeFs.is_dir(path)
    }

    fn canonical(&self, path: &Path) -> PathBuf {
        self.canonical_calls.set(self.canonical_calls.get() + 1);
        jaic::sema::NativeFs.canonical(path)
    }

    fn canonical_exact(&self, path: &Path) -> Option<PathBuf> {
        jaic::sema::NativeFs.canonical_exact(path)
    }

    fn missing_file_key(&self, path: &Path) -> Option<PathBuf> {
        if !self.enabled.get() {
            return None;
        }
        if self.create_at.borrow().as_deref() == Some(path) {
            self.create_at.borrow_mut().take();
            std::fs::write(path, "answer :: 2;").unwrap();
        }
        let key = jaic::sema::NativeFs.missing_file_key(path);
        if self.stop_at.borrow().as_deref() == Some(path) {
            self.stop_at.borrow_mut().take();
            jaic::interrupt::request(true);
            assert!(jaic::interrupt::should_stop());
        }
        key
    }

    fn list_dir(&self, path: &Path) -> Vec<(String, bool)> {
        jaic::sema::NativeFs.list_dir(path)
    }

    fn stamp(&self, path: &Path) -> Option<u64> {
        jaic::sema::NativeFs.stamp(path)
    }
}

fn native_overlay(base: Rc<dyn FileSystem>, root: &Path) -> OverlayFs {
    OverlayFs {
        base,
        dir: root.to_path_buf(),
        files: RefCell::default(),
        generated: BTreeMap::default(),
        canonical: jaic::paths::CanonCache::default(),
        reads: Rc::default(),
    }
}

#[test]
fn missing_file_probe_certifies_only_exact_native_absence() {
    let dir = test_support::TempDir::new("missing-probe-law");
    let absent = dir.join("absent/module.jai");
    let native = jaic::sema::NativeFs;
    assert_eq!(native.canonical_exact(&absent), None);
    assert_eq!(
        native.missing_file_key(&absent),
        Some(jaic::paths::file_identity(&native, &absent))
    );
    assert_eq!(
        native.missing_file_key(dir.path()),
        None,
        "a directory is not resolving absence"
    );
    let file = dir.write("existing.jai", "main :: () {}");
    assert_eq!(native.missing_file_key(&file), None);
    let blocked = file.join("module.jai");
    assert!(matches!(
        std::fs::metadata(&blocked).unwrap_err().kind(),
        std::io::ErrorKind::NotFound | std::io::ErrorKind::NotADirectory
    ));
    assert_eq!(
        native.missing_file_key(&blocked),
        Some(jaic::paths::file_identity(&native, &blocked))
    );
    assert_eq!(
        VirtualFs::default().missing_file_key(&absent),
        None,
        "unsupported providers cannot certify absence"
    );
}

#[test]
fn missing_file_candidate_avoids_canonical_work_and_negative_path_cache() {
    let dir = test_support::TempDir::new("missing-probe-capture");
    let base = Rc::new(MissingProbeFs::new());
    let fs = native_overlay(base.clone(), dir.path());
    let path = dir.join("absent/module.jai");
    assert!(!fs.is_file(&path));
    assert_eq!(
        base.canonical_calls.get(),
        0,
        "an uncached negative file probe needs no canonical/ancestor requests"
    );
    assert!(
        fs.canonical.get(&path).is_none(),
        "negative results cannot become canonical cache entries"
    );
    assert!(
        fs.reads
            .borrow()
            .concerns(&jaic::paths::file_identity(&jaic::sema::NativeFs, &path))
    );
    assert!(fs.reads.borrow().is_current(&*base));
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(&path, "created").unwrap();
    assert!(
        !fs.reads.borrow().is_current(&*base),
        "higher-priority candidate creation changes exact absence"
    );
}

#[test]
fn missing_probe_preserves_unsaved_and_generated_overlay_precedence() {
    let dir = test_support::TempDir::new("missing-probe-open");
    for generated in [false, true] {
        let base = Rc::new(MissingProbeFs::new());
        let mut fs = native_overlay(base.clone(), dir.path());
        let path = dir.join("not-saved.jai");
        let key = jaic::paths::file_identity(&jaic::sema::NativeFs, &path);
        if generated {
            fs.generated.insert(key, Arc::from(&b"generated"[..]));
        } else {
            fs.files.get_mut().insert(key, Arc::from(&b"open"[..]));
        }
        assert!(fs.is_file(&path));
        assert_eq!(
            fs.read(&path).unwrap(),
            if generated {
                &b"generated"[..]
            } else {
                &b"open"[..]
            }
        );
        assert!(
            base.canonical_calls.get() > 0,
            "overlay hits preserve the existing identity path"
        );
    }
}

#[test]
fn missing_probe_unavailability_and_not_directory_transition_reject_old_proof() {
    let dir = test_support::TempDir::new("missing-probe-transition");
    let base = Rc::new(MissingProbeFs::new());
    let fs = native_overlay(base.clone(), dir.path());
    let blocker = dir.write("blocker", "not a directory");
    let candidate = blocker.join("module.jai");
    assert!(!fs.is_file(&candidate));
    assert!(fs.reads.borrow().is_current(&*base));
    base.enabled.set(false);
    assert!(
        !fs.reads.borrow().is_current(&*base),
        "a provider cannot silently withdraw exact authority"
    );
    base.enabled.set(true);
    std::fs::remove_file(&blocker).unwrap();
    std::fs::create_dir(&blocker).unwrap();
    std::fs::write(&candidate, "now present").unwrap();
    assert!(!fs.reads.borrow().is_current(&*base));
}

#[test]
fn missing_probe_unknown_provider_preserves_the_original_path() {
    let dir = test_support::TempDir::new("missing-probe-unknown");
    let base = Rc::new(MissingProbeFs::new());
    base.enabled.set(false);
    let fs = native_overlay(base.clone(), dir.path());
    let path = dir.join("not-present.jai");
    assert!(!fs.is_file(&path));
    assert!(base.canonical_calls.get() > 0);
    assert!(fs.canonical.get(&path).is_some());
    assert!(fs.reads.borrow().is_current(&*base));
}

#[cfg(unix)]
#[test]
fn missing_probe_parent_retarget_preserves_absence_but_rejects_a_new_candidate() {
    let dir = test_support::TempDir::new("missing-probe-retarget");
    std::fs::create_dir(dir.join("first")).unwrap();
    std::fs::create_dir(dir.join("second")).unwrap();
    let alias = dir.join("alias");
    std::os::unix::fs::symlink(dir.join("first"), &alias).unwrap();
    let fs = native_overlay(Rc::new(jaic::sema::NativeFs), dir.path());
    let candidate = alias.join("Module.jai");
    assert!(!fs.is_file(&candidate));
    assert!(fs.reads.borrow().is_current(&*fs.base));
    std::fs::remove_file(&alias).unwrap();
    std::os::unix::fs::symlink(dir.join("second"), &alias).unwrap();
    assert!(
        fs.reads.borrow().is_current(&*fs.base),
        "retargeting one absent choice to another absent choice preserves false"
    );
    std::fs::write(dir.join("second/Module.jai"), "now present").unwrap();
    assert!(!fs.reads.borrow().is_current(&*fs.base));
}

#[cfg(unix)]
#[test]
fn missing_probe_cannot_replace_a_cached_deleted_alias_open_target() {
    let dir = test_support::TempDir::new("missing-probe-cached-alias");
    let target = dir.write("target.jai", "disk");
    let alias = dir.join("alias.jai");
    std::os::unix::fs::symlink(&target, &alias).unwrap();
    let key = test_support::lsp::native_identity(&target);
    let mut fs = native_overlay(Rc::new(jaic::sema::NativeFs), dir.path());
    fs.files
        .get_mut()
        .insert(key.clone(), Arc::from(&b"open"[..]));
    assert_eq!(fs.canonical(&alias), key);
    std::fs::remove_file(&target).unwrap();
    assert!(jaic::sema::NativeFs.missing_file_key(&alias).is_some());
    assert!(
        fs.is_file(&alias),
        "the prior canonical key still selects the open target"
    );
    assert_eq!(fs.read(&alias).unwrap(), b"open");
    assert!(
        !fs.reads.borrow().is_current(&*fs.base),
        "the old canonical history is retained and rejects the changed disk alias"
    );
}

#[cfg(unix)]
#[test]
fn missing_probe_keeps_other_canonical_producer_history_sticky() {
    let dir = test_support::TempDir::new("missing-probe-coalesced");
    std::fs::create_dir(dir.join("first")).unwrap();
    std::fs::create_dir(dir.join("second")).unwrap();
    let alias = dir.join("alias");
    std::os::unix::fs::symlink(dir.join("first"), &alias).unwrap();
    let fs = native_overlay(Rc::new(jaic::sema::NativeFs), dir.path());
    assert_eq!(
        fs.canonical(&alias),
        test_support::lsp::native_identity(&dir.join("first"))
    );
    assert!(!fs.is_file(&alias.join("Missing.jai")));
    std::fs::remove_file(&alias).unwrap();
    std::os::unix::fs::symlink(dir.join("second"), &alias).unwrap();
    assert!(
        !fs.reads.borrow().is_current(&*fs.base),
        "a new absence cannot erase another producer's canonical authority"
    );
}

fn missing_module_session(
    name: &str,
) -> (
    test_support::TempDir,
    crate::Session,
    Rc<MissingProbeFs>,
    crate::DocumentUri,
    PathBuf,
    String,
) {
    let dir = test_support::TempDir::new(name);
    let project = dir.path().join("workspace/project/source");
    let first = dir.path().join("first");
    let second = dir.path().join("second");
    for path in [&project, &first, &second] {
        std::fs::create_dir_all(path).unwrap();
    }
    let first = test_support::lsp::native_identity(&first);
    let second = test_support::lsp::native_identity(&second);
    std::fs::write(second.join("Library.jai"), "answer :: 1;").unwrap();
    let source = "#import \"Library\"; main :: () { local := answer; local += 1; }".to_owned();
    let main = project.join("main.jai");
    std::fs::write(&main, &source).unwrap();
    let uri = crate::DocumentUri::from_path(&main).unwrap();
    let candidate = first.join("Library.jai");
    let base = Rc::new(MissingProbeFs::new());
    let mut session = crate::Session::with_environment(
        crate::Limits::default(),
        crate::Environment {
            fs: base.clone(),
            services: None,
            options: Box::new(move |_, _| {
                let mut options = jaic::sema::Options::host();
                options.runtime_support = false;
                options.stack_trace = false;
                options.import_paths = vec![first.clone(), second.clone()];
                options
            }),
        },
    );
    session.open(uri.clone(), 1, source.clone()).unwrap();
    assert!(session.diagnostics(&uri).unwrap().is_empty());
    (dir, session, base, uri, candidate, source)
}

#[test]
fn missing_probe_new_unsaved_higher_priority_module_invalidates_the_old_choice() {
    let (_dir, mut session, _base, uri, candidate, source) =
        missing_module_session("missing-probe-overlay-selection");
    let at = test_support::lsp::at_nth(&source, "answer", 0, 1);
    let before = session.definition(&uri, at).unwrap();
    assert_eq!(before.len(), 1);
    let upper = crate::DocumentUri::from_path(&candidate).unwrap();
    assert_ne!(before[0].uri, upper.as_str());
    assert!(!candidate.exists());
    session
        .open(upper.clone(), 1, "answer :: 2;".into())
        .unwrap();
    let after = session.definition(&uri, at).unwrap();
    assert_eq!(after.len(), 1);
    assert_eq!(
        after[0].uri,
        upper.as_str(),
        "absence concerns must notice a newly opened effective fallback key"
    );
}

#[test]
fn missing_probe_selected_failure_is_terminal_and_later_stable_query_recovers() {
    let (_dir, session, base, uri, candidate, source) =
        missing_module_session("missing-probe-selected-failure");
    let at = test_support::lsp::at_nth(&source, "local", 0, 1);
    let expected = session.references(&uri, at, true).unwrap();
    assert_eq!(expected.len(), 2);
    let counts = session.compile_counts();
    // Exercise native capture admission; a warm accepted epoch intentionally has no native replay.
    session.input_epoch.as_ref().unwrap().begin_capture();
    *base.create_at.borrow_mut() = Some(candidate);
    assert!(session.references(&uri, at, true).unwrap().is_empty());
    assert!(
        base.create_at.borrow().is_none(),
        "the exact absence replay was actually reached"
    );
    assert_eq!(
        session.compile_counts(),
        counts,
        "selected proof failure cannot trigger a second provider/compile"
    );
    assert_eq!(session.references(&uri, at, true).unwrap(), expected);
    assert_eq!(session.compile_counts().0, counts.0 + 1);
}

#[test]
fn missing_probe_replay_interruption_cannot_publish_an_answer() {
    let (_dir, session, base, uri, candidate, source) =
        missing_module_session("missing-probe-cancel");
    let at = test_support::lsp::at_nth(&source, "local", 0, 1);
    let expected = session.references(&uri, at, true).unwrap();
    assert_eq!(expected.len(), 2);
    let counts = session.compile_counts();
    // Exercise native capture admission; a warm accepted epoch intentionally has no native replay.
    session.input_epoch.as_ref().unwrap().begin_capture();
    *base.stop_at.borrow_mut() = Some(candidate);
    jaic::interrupt::arm();
    let result = session.references(&uri, at, true).unwrap();
    let interrupted = jaic::interrupt::disarm();
    jaic::interrupt::request(false);
    assert!(interrupted && base.stop_at.borrow().is_none());
    assert!(result.is_empty());
    assert_eq!(session.compile_counts(), counts);
    assert_eq!(session.references(&uri, at, true).unwrap(), expected);
}

#[test]
fn missing_probe_invalid_input_is_unknown_not_certified_absence() {
    let path = Path::new("invalid\0name");
    assert!(std::fs::metadata(path).is_err());
    assert_eq!(jaic::sema::NativeFs.missing_file_key(path), None);
}

#[cfg(unix)]
#[test]
fn missing_probe_symlink_loop_is_unknown_not_certified_absence() {
    let dir = test_support::TempDir::new("missing-probe-loop");
    let alias = dir.join("loop");
    std::os::unix::fs::symlink(&alias, &alias).unwrap();
    assert!(std::fs::metadata(&alias).is_err());
    assert_eq!(jaic::sema::NativeFs.missing_file_key(&alias), None);
}

#[test]
fn missing_probe_records_raw_relative_and_normalized_fallback_concerns() {
    let dir = test_support::TempDir::new("missing-probe-raw-concerns");
    let base = Rc::new(MissingProbeFs::new());
    let fs = native_overlay(base, dir.path());
    let relative = Path::new("absent/../candidate.jai");
    let raw = dir.join(relative);
    let fallback = jaic::paths::file_identity(&jaic::sema::NativeFs, &raw);
    assert_ne!(
        raw, fallback,
        "the fixture must exercise distinct raw and fallback spellings"
    );
    assert!(!fs.is_file(relative));
    let reads = fs.reads.borrow();
    assert!(reads.concerns(&raw));
    assert!(reads.concerns(&fallback));
    assert!(reads.concerns_tree(dir.path()));
}
