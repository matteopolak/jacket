use super::*;

fn fixture() -> (Cache, Environment, PathBuf, Buffers, Buffers) {
    let env = Environment {
        services: None,
        fs: Rc::new(jaic::sema::NativeFs),
        options: Box::new(|_, _| {
            let mut options = jaic::sema::Options::host();
            options.runtime_support = false;
            options.stack_trace = false;
            options
        }),
    };
    let root = PathBuf::from("/cache-collision/main.jai");
    let files = |text: &[u8]| Rc::new(BTreeMap::from([(root.clone(), Arc::from(text))]));
    let old = files(b"answer :: 1; main :: () {}");
    let new = files(b"answer :: 2; main :: () {}");
    let mut cache = Cache::default();
    cache
        .analyze(&env, &root, &[], old.clone())
        .expect("checked fixture");
    (cache, env, root, old, new)
}

#[test]
fn selecting_one_program_validates_only_its_native_dependencies() {
    use jaic::sema::{FileSystem, NativeFs};
    struct CountingFs {
        watched: PathBuf,
        reads: std::cell::Cell<usize>,
    }
    impl FileSystem for CountingFs {
        fn read(&self, path: &Path) -> Option<Vec<u8>> {
            if path == self.watched {
                self.reads.set(self.reads.get() + 1);
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
        fn canonical_exact(&self, path: &Path) -> Option<PathBuf> {
            NativeFs.canonical_exact(path)
        }
        fn list_dir(&self, path: &Path) -> Vec<(String, bool)> {
            NativeFs.list_dir(path)
        }
        fn stamp(&self, path: &Path) -> Option<u64> {
            NativeFs.stamp(path)
        }
        fn is_native(&self) -> bool {
            true
        }
    }
    let temp = test_support::TempDir::new("selected-proof");
    let root = temp.path().join("workspace/project/source");
    let a = root.join("a/main.jai");
    let b = root.join("b/main.jai");
    std::fs::create_dir_all(a.parent().unwrap()).unwrap();
    std::fs::create_dir_all(b.parent().unwrap()).unwrap();
    let dependency = b.parent().unwrap().join("dependency.jai");
    std::fs::write(&dependency, "provided :: 1;").unwrap();
    let fs = Rc::new(CountingFs {
        watched: test_support::lsp::native_identity(&dependency),
        reads: std::cell::Cell::new(0),
    });
    let env = Environment {
        services: None,
        fs: fs.clone(),
        options: Box::new(|_, _| {
            let mut options = jaic::sema::Options::host();
            options.runtime_support = false;
            options.stack_trace = false;
            options
        }),
    };
    let files = Rc::new(BTreeMap::from([
        (a.clone(), Arc::<[u8]>::from(&b"main :: () {}"[..])),
        (
            b.clone(),
            Arc::<[u8]>::from(&b"#load \"dependency.jai\"; main :: () { x := provided; }"[..]),
        ),
    ]));
    let mut cache = Cache::default();
    cache.analyze(&env, &a, &[], files.clone()).unwrap();
    cache.analyze(&env, &b, &[], files.clone()).unwrap();
    assert!(
        fs.reads.get() > 0,
        "second program really reads its dependency"
    );
    fs.reads.set(0);
    assert!(cache.cached(&env, &a, &[], &files).is_some());
    assert_eq!(
        fs.reads.get(),
        0,
        "an unrelated retained program cannot add native I/O to this hit"
    );
    std::fs::write(&dependency, "provided :: 2;").unwrap();
    assert!(
        cache.cached(&env, &b, &[], &files).is_none(),
        "selecting the changed program must still validate its exact proof"
    );
}

#[test]
fn a_content_hash_collision_cannot_reuse_other_source_text() {
    let (mut cache, env, root, _, new) = fixture();
    // Force the candidate bucket a real hash collision would select. Source equality,
    // rather than the hash function's accidental uniqueness, must establish reuse.
    let inputs = cache.inputs(&env, &root, &[], &new, None);
    cache.entries[0].key = inputs.key;
    cache.entries[0].quick = None;
    assert!(cache.cached(&env, &root, &[], &new).is_none());
}

#[test]
fn a_quick_hash_collision_cannot_reuse_another_buffer_map() {
    let (mut cache, env, root, _, new) = fixture();
    cache.entries[0].quick.as_mut().unwrap().0 = quick_key(&root, &[], &new, None);
    assert!(cache.cached(&env, &root, &[], &new).is_none());
}

#[test]
fn colliding_keys_cannot_change_program_import_roots_or_target() {
    for changed in 0..3 {
        let (mut cache, env, root, old, _) = fixture();
        let requested_root = if changed == 0 {
            PathBuf::from("/cache-collision/other.jai")
        } else {
            root
        };
        let dirs = if changed == 1 {
            vec![PathBuf::from("/other-modules")]
        } else {
            vec![]
        };
        let os = (changed == 2).then_some(TargetOs::Linux);
        let inputs = cache.inputs(&env, &requested_root, &dirs, &old, os);
        cache.entries[0].key = inputs.key;
        cache.entries[0].quick.as_mut().unwrap().0 = inputs.quick;
        assert!(
            cache
                .lookup(&env, &requested_root, &dirs, &old, os)
                .is_err()
        );
    }
}

/// Manual component measurement; run its built release test binary under an exclusive lease.
/// Example: `cargo test --release -p jai-language-server --lib native_cache_validation_benchmark -- --ignored --nocapture --test-threads=1`.
#[test]
#[ignore = "manual measurement requires an exclusive quiet-machine lease"]
fn native_cache_validation_benchmark() {
    let repository = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .unwrap();
    let stdlib = repository.join("stdlib");
    for (relative, declaration) in [
        ("tools/bench/fixtures/lsp-small/main.jai", "twice ::"),
        ("examples/kvdb/main.jai", "run_local ::"),
        ("examples/editor/editor.jai", "status_line ::"),
    ] {
        let path = repository.join(relative);
        let text = std::fs::read_to_string(&path).unwrap();
        let offset = text.find(declaration).expect("known nonempty target");
        let files: Buffers = Rc::new([(path.clone(), Arc::from(text.as_bytes()))].into());
        let stdlib = stdlib.clone();
        let env = Environment {
            fs: Rc::new(jaic::sema::NativeFs),
            services: None,
            options: Box::new(move |fs, main| {
                let inputs = jaic::project::ProgramInputs::for_file(fs, main, &stdlib);
                jaic::sema::Options::for_program(fs, main, &inputs)
            }),
        };
        let mut cache = Cache::default();
        let analysis = cache.analyze(&env, &path, &[], files.clone()).unwrap();
        analysis.focus(&path);
        let coverage = analysis.coverage(&path);
        assert!(
            coverage.source_loaded && coverage.program_complete,
            "guard workload must check completely: {relative}"
        );
        let file = analysis.file(&path).unwrap();
        let references = analysis.compiler.ide_references(file, offset as u32);
        assert!(!references.is_empty());
        let proof_bytes = analysis.reads.borrow().retained_bytes();
        println!(
            "file={relative} oracle_results={} input_proof_bytes={proof_bytes}",
            references.len()
        );
        for sample in 0..7 {
            let start = std::time::Instant::now();
            let present = cache.cached(&env, &path, &[], &files).is_some();
            let elapsed = start.elapsed();
            assert!(present, "must time an accepted cache hit");
            println!(
                "sample={sample} native_cached_input_ms={:.6}",
                elapsed.as_secs_f64() * 1000.0
            );
        }
    }
}

#[test]
fn interruption_during_final_replay_cannot_accept_a_build_or_notice() {
    use jaic::sema::{FileSystem, NativeFs};
    struct StopFs {
        dependency: PathBuf,
        reads: std::cell::Cell<usize>,
    }
    impl FileSystem for StopFs {
        fn is_native(&self) -> bool {
            true
        }
        fn read(&self, path: &Path) -> Option<Vec<u8>> {
            let bytes = NativeFs.read(path);
            if path == self.dependency {
                let count = self.reads.get() + 1;
                self.reads.set(count);
                if count == 2 {
                    jaic::interrupt::request(true);
                    assert!(jaic::interrupt::should_stop());
                }
            }
            bytes
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
        fn canonical(&self, path: &Path) -> PathBuf {
            NativeFs.canonical(path)
        }
        fn canonical_exact(&self, path: &Path) -> Option<PathBuf> {
            NativeFs.canonical_exact(path)
        }
    }
    let dir = test_support::TempDir::new("notice-final-replay-cancel");
    let root = dir.path().join("main.jai");
    let dependency = dir.path().join("dependency.jai");
    let source =
        "#load \"dependency.jai\"; spin :: () { while true {} }\n#run spin(); main :: () {}";
    std::fs::write(&root, source).unwrap();
    std::fs::write(&dependency, "helper :: 1;").unwrap();
    let root = test_support::lsp::native_identity(&root);
    let dependency = test_support::lsp::native_identity(&dependency);
    let fs = Rc::new(StopFs {
        dependency,
        reads: std::cell::Cell::new(0),
    });
    let env = Environment {
        fs: fs.clone(),
        services: None,
        options: Box::new(|_, _| {
            let mut options = jaic::sema::Options::host();
            options.runtime_support = false;
            options.stack_trace = false;
            options
        }),
    };
    let mut cache = Cache::default();
    cache.set_limit_notices(true);
    cache.set_limits(CheckLimits::new(16, 0.0));
    let files = Rc::new(BTreeMap::from([(
        root.clone(),
        Arc::from(source.as_bytes()),
    )]));
    jaic::interrupt::arm();
    let accepted = cache.analyze(&env, &root, &[], files).is_some();
    let interrupted = jaic::interrupt::disarm();
    jaic::interrupt::request(false);
    assert!(
        interrupted && fs.reads.get() >= 2,
        "must interrupt during the native replay"
    );
    assert!(
        !accepted,
        "final replay interruption cannot accept an Analysis"
    );
    assert_eq!(cache.limit_notice_counts(), (0, 0, 0));
}

#[cfg(unix)]
#[test]
fn producer_key_identity_is_observed_before_the_build_acceptance_guard() {
    producer_key_projection(false, false);
}

#[cfg(unix)]
#[test]
fn explicit_probe_key_identity_is_observed_before_publication() {
    producer_key_projection(true, false);
}

#[cfg(unix)]
#[test]
fn registry_input_selection_keeps_transient_root_identity_responses() {
    producer_key_projection(false, true);
}

#[cfg(unix)]
fn producer_key_projection(probe: bool, temporary: bool) {
    use jaic::sema::{FileSystem, NativeFs};
    use std::cell::Cell;
    struct RetargetFs {
        alias: PathBuf,
        second: PathBuf,
        first: PathBuf,
        temporary: bool,
        calls: Cell<usize>,
        retarget_at: Cell<Option<usize>>,
        changed: Cell<bool>,
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
            if path == self.alias.join("main.jai") {
                let call = self.calls.get() + 1;
                self.calls.set(call);
                if self.retarget_at.get() == Some(call) {
                    self.retarget_at.set(None);
                    self.changed.set(true);
                    std::fs::remove_file(&self.alias).unwrap();
                    std::os::unix::fs::symlink(&self.second, &self.alias).unwrap();
                    if self.temporary {
                        let changed = NativeFs.canonical(path);
                        std::fs::remove_file(&self.alias).unwrap();
                        std::os::unix::fs::symlink(&self.first, &self.alias).unwrap();
                        return changed;
                    }
                }
            }
            NativeFs.canonical(path)
        }
    }
    let dir = test_support::TempDir::new("producer-key-projection");
    let first = dir.path().join("first");
    let second = dir.path().join("second");
    let alias = dir.path().join("alias");
    let source = r#"#import "Compiler";
#run {
set_build_options_dc(.{do_output=false});
child := compiler_create_workspace("child");
options := get_build_options(child); options.output_type=.NO_OUTPUT; set_build_options(options,child);
add_build_string("provided :: 42;", child);
}
"#;
    for folder in [&first, &second] {
        std::fs::create_dir(folder).unwrap();
        std::fs::write(folder.join("main.jai"), source).unwrap();
    }
    std::os::unix::fs::symlink(&first, &alias).unwrap();
    let fs = Rc::new(RetargetFs {
        alias: alias.clone(),
        second,
        first: first.clone(),
        temporary,
        calls: Cell::new(0),
        retarget_at: Cell::new(None),
        changed: Cell::new(false),
    });
    let stdlib = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../stdlib");
    let env = Environment {
        fs: fs.clone(),
        services: None,
        options: Box::new(move |fs, _| {
            let mut options = jaic::sema::Options::host();
            options.stack_trace = false;
            options.import_paths = vec![stdlib.clone()];
            options.set_stdlib(fs, &stdlib);
            options
        }),
    };
    let root = alias.join("main.jai");
    let files = Rc::new(BTreeMap::from([(
        root.clone(),
        Arc::from(source.as_bytes()),
    )]));
    let build = || {
        let mut cache = Cache::default();
        let accepted = if probe {
            cache.probe(&env, &root, &[], files.clone()).is_some()
        } else {
            cache.analyze(&env, &root, &[], files.clone()).is_some()
        };
        (accepted, cache.recorded.len())
    };
    assert_eq!(build(), (true, 1));
    fs.calls.set(0);
    assert_eq!(build(), (true, 1));
    let calls = fs.calls.get();
    assert!(calls > 0);
    fs.calls.set(0);
    assert_eq!(build(), (true, 1));
    assert_eq!(fs.calls.get(), calls, "stable fresh-build control");
    if temporary {
        for call in 1..=calls {
            fs.calls.set(0);
            fs.changed.set(false);
            fs.retarget_at.set(Some(call));
            let found = build();
            assert!(fs.changed.get());
            assert_eq!(
                found,
                (false, 0),
                "actual transient canonical response at call {call} must join the accepted history, including the negative registry selection before compilation"
            );
        }
        assert_eq!(
            build(),
            (true, 1),
            "later stable root identity remains useful"
        );
        return;
    }
    fs.calls.set(0);
    fs.retarget_at.set(Some(calls));
    let found = build();
    assert!(fs.changed.get());
    assert_eq!(
        found,
        (false, usize::from(probe)),
        "a producer key cannot be chosen from a different postguard native identity; an earlier accepted build record stays owned by its original context"
    );
    assert_eq!(
        build(),
        (true, 1),
        "later stable request can accept the new producer context"
    );
}

struct PublishedFixture {
    cache: Cache,
    root: PathBuf,
    files: Buffers,
    owners: [Rc<()>; 2],
    indexes: [Rc<jaic::ide::meta::ReferenceIndex>; 2],
    _directory: test_support::TempDir,
}

fn published_fixture() -> PublishedFixture {
    let directory = test_support::lsp::project(
        "published-candidate-order",
        &[("main.jai", "main :: () {}")],
    );
    let root = directory.join("main.jai");
    let files: Buffers = Rc::new([(root.clone(), Arc::from(&b"main :: () {}"[..]))].into());
    let supplied = test_support::lsp::environment();
    let environment = Environment {
        fs: supplied.fs,
        options: supplied.options,
        services: supplied.services,
    };
    let mut cache = Cache::default();
    let first = cache
        .analyze(&environment, &root, &[], files.clone())
        .unwrap();
    let first_index = first.reference_index();
    let first_owner = first.reference_owner.clone();
    let mut second = cache
        .compile_detached(&environment, &root, &[], files.clone())
        .unwrap();
    let second_index = second.reference_index();
    let second_owner = second.reference_owner.clone();
    cache.entries.push(second);
    PublishedFixture {
        cache,
        root,
        files,
        owners: [first_owner, second_owner],
        indexes: [first_index, second_index],
        _directory: directory,
    }
}

#[test]
fn published_views_and_retirement_use_only_the_first_exact_candidate() {
    let mut fixture = published_fixture();
    let query = |cache: &Cache, owner: usize| {
        cache.published_reference_inputs(
            &fixture.root,
            &[],
            &fixture.files,
            &fixture.owners[owner],
            &fixture.indexes[owner],
        )
    };
    assert!(query(&fixture.cache, 0).is_some());
    assert!(
        query(&fixture.cache, 1).is_none(),
        "a later matching owner cannot replace normal cache selection"
    );
    fixture.cache.entries[0].facts_accepted.set(false);
    assert!(
        query(&fixture.cache, 1).is_none(),
        "an unaccepted first candidate cannot authorize searching past it"
    );
    fixture.cache.entries[0].facts_accepted.set(true);
    fixture.cache.retire_published_reference_view(
        &fixture.root,
        &[],
        &fixture.files,
        &fixture.owners[1],
        &fixture.indexes[1],
    );
    assert!(fixture.cache.entries.iter().all(|entry| entry.query_valid));
    fixture.cache.retire_published_reference_view(
        &fixture.root,
        &[],
        &fixture.files,
        &fixture.owners[0],
        &fixture.indexes[0],
    );
    assert!(!fixture.cache.entries[0].query_valid && fixture.cache.entries[1].query_valid);
    assert!(
        query(&fixture.cache, 1).is_some(),
        "later requests may select the next valid candidate"
    );
}

#[test]
fn published_views_preserve_current_program_precedence_for_shared_sources() {
    let mut fixture = published_fixture();
    // Assemble another contained Program from a real independent compile of the same
    // source. focus(root) must keep the current Program when both contain that file.
    let other = fixture.cache.entries.pop().unwrap();
    fixture.cache.entries[0].others.push(super::super::Program {
        compiler: other.compiler,
        entry: other.entry,
        complete: other.complete,
        errors: other.errors,
        lints: other.lints,
        references: other.references,
        reference_owner: other.reference_owner,
    });
    fixture.cache.entries[0].focus(&fixture.root);
    assert!(Rc::ptr_eq(
        &fixture.cache.entries[0].reference_owner,
        &fixture.owners[0]
    ));
    assert!(
        fixture
            .cache
            .published_reference_inputs(
                &fixture.root,
                &[],
                &fixture.files,
                &fixture.owners[0],
                &fixture.indexes[0]
            )
            .is_some()
    );
    assert!(
        fixture
            .cache
            .published_reference_inputs(
                &fixture.root,
                &[],
                &fixture.files,
                &fixture.owners[1],
                &fixture.indexes[1]
            )
            .is_none(),
        "a shared source in another Program cannot bypass current-focus precedence"
    );
}
