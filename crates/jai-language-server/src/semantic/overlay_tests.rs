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
