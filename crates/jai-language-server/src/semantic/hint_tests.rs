//! Render-time dependencies must not become compiler-resolution authority.
use super::*;
use serde_json::{Value, json};

fn fixture() -> (
    test_support::TempDir,
    Environment,
    PathBuf,
    PathBuf,
    Buffers,
) {
    let temp = test_support::TempDir::new("owned-name-hints");
    let directory = temp.path().join("project/src");
    let modules = temp.path().join("modules");
    std::fs::create_dir_all(&directory).unwrap();
    std::fs::create_dir_all(modules.join("Widgets")).unwrap();
    let module = modules.join("Widgets/module.jai");
    std::fs::write(&module, "Wanted :: 42;").unwrap();
    let main = directory.join("main.jai");
    let text = "helper :: (item: int) -> int { return item+1; }\nmain :: () { value := Wanted; }";
    std::fs::write(&main, text).unwrap();
    let files = Rc::new([(main.clone(), Arc::from(text.as_bytes()))].into());
    let env = Environment {
        fs: Rc::new(jaic::sema::NativeFs),
        services: None,
        options: Box::new(move |_, _| {
            let mut options = Options::host();
            options.runtime_support = false;
            options.stack_trace = false;
            options.preload_stdlib = Some(modules.clone());
            options.import_paths = vec![modules.clone()];
            options
        }),
    };
    (temp, env, main, module, files)
}

fn findings(analysis: &mut Analysis, path: &std::path::Path) -> Vec<super::queries::CheckFinding> {
    let _query = analysis.hint_query();
    let findings = analysis.staged_check_findings(path);
    assert!(analysis.reads.borrow().is_current(&*analysis.overlay.base));
    assert!(analysis.hints_current(&*analysis.overlay.base));
    findings
}

#[test]
fn discarded_dead_branch_operator_hints_do_not_become_semantic_inputs() {
    let (_temp, env, main, module, _) = fixture();
    let text = "Thing :: struct { field: int; }\nmain :: () { if false { left: Thing; right: Thing; value := left + right; } }";
    std::fs::write(&main, text).unwrap();
    let files: Buffers = Rc::new([(main.clone(), Arc::from(text.as_bytes()))].into());
    let mut cache = Cache::default();
    let analysis = cache.analyze(&env, &main, &[], files.clone()).unwrap();
    assert!(
        analysis.errors.is_empty(),
        "constant-dead errors remain discarded"
    );
    assert!(analysis.coverage(&main).program_complete);
    let file = analysis.file(&main).unwrap();
    assert_eq!(
        analysis
            .compiler
            .ide_references(file, text.find("left:").unwrap() as u32)
            .len(),
        2,
        "checking the dead branch must still retain useful editor facts"
    );
    let modified = std::fs::metadata(&module).unwrap().modified().unwrap();
    std::fs::write(&module, "Absent :: 42;").unwrap();
    std::fs::File::options()
        .write(true)
        .open(&module)
        .unwrap()
        .set_times(std::fs::FileTimes::new().set_modified(modified))
        .unwrap();
    assert!(
        cache.cached(&env, &main, &[], &files).is_some(),
        "a discarded import hint must not make an unrelated stdlib module semantic authority"
    );
    assert_eq!(cache.counts().0, 1);
}

#[test]
fn deferred_operator_help_is_idempotent_and_not_subject_to_unknown_name_cap() {
    let (_temp, env, main, module, _) = fixture();
    std::fs::write(&module, "Thing :: struct { field: int; }\noperator + :: (a: Thing, b: Thing) -> Thing { return a; }").unwrap();
    let text = "Thing :: struct { field: int; }\nmain :: () { left: Thing; right: Thing; value := left + right; }";
    std::fs::write(&main, text).unwrap();
    let files: Buffers = Rc::new([(main.clone(), Arc::from(text.as_bytes()))].into());
    let mut cache = Cache::default();
    let analysis = cache.analyze(&env, &main, &[], files).unwrap();
    let raw = analysis
        .errors
        .iter()
        .find(|d| d.operator_import_hint().is_some())
        .unwrap()
        .clone();
    assert!(
        raw.suggestions().is_empty(),
        "external fixes are pending, not checked facts"
    );
    let observer = analysis.overlay.separate_observer();
    let context = jaic::sema::HintContext::default();
    let prepared = analysis
        .compiler
        .with_name_suggestion_in(&raw, &context, &observer)
        .unwrap();
    assert!(prepared.operator_import_hint().is_none());
    assert!(
        prepared
            .suggestions()
            .iter()
            .any(|s| s.title.contains("Widgets"))
    );
    assert!(
        analysis
            .compiler
            .with_name_suggestion_in(&prepared, &context, &observer)
            .is_none(),
        "prepared help cannot be duplicated"
    );
    assert!(
        raw.operator_import_hint().is_some(),
        "the original remains renderable"
    );
    let mut cleared = raw.clone();
    cleared.clear_suggestions();
    assert!(
        analysis
            .compiler
            .with_name_suggestion_in(&cleared, &context, &observer)
            .is_none(),
        "clearing fixes also prevents deferred fixes from reappearing"
    );
    analysis.errors = vec![
        jaic::diagnostics::Diagnostic::new(
            jaic::diagnostics::Code::UnknownIdentifier,
            raw.span,
            "padding"
        );
        50
    ];
    analysis.errors.push(raw);
    assert!(
        findings(analysis, &main)
            .iter()
            .any(|f| f.display.contains("Widgets")),
        "operator help was already available beyond the unknown-name enrichment cap"
    );
}

#[test]
fn name_hint_enrichment_keeps_its_files_out_of_semantic_proof() {
    let (_temp, env, main, _module, files) = fixture();
    let mut cache = Cache::default();
    let analysis = cache.analyze(&env, &main, &[], files).unwrap();
    let before = analysis.reads.borrow().retained_bytes();
    let findings = findings(analysis, &main);
    assert!(
        findings
            .iter()
            .any(|finding| finding.display.contains("Widgets")),
        "expected useful import hint"
    );
    assert_eq!(
        analysis.reads.borrow().retained_bytes(),
        before,
        "render-only reads need separately owned proof; unrelated navigation must not inherit them"
    );
}

#[test]
fn silent_hint_rewrite_refreshes_help_without_retiring_the_semantic_compiler() {
    let (_temp, env, main, module, files) = fixture();
    let mut cache = Cache::default();
    let analysis = cache.analyze(&env, &main, &[], files.clone()).unwrap();
    assert!(
        findings(analysis, &main)
            .iter()
            .any(|finding| finding.display.contains("Widgets"))
    );
    let old = std::fs::metadata(&module).unwrap().modified().unwrap();
    std::fs::write(&module, "Absent :: 42;").unwrap();
    std::fs::File::options()
        .write(true)
        .open(&module)
        .unwrap()
        .set_times(std::fs::FileTimes::new().set_modified(old))
        .unwrap();
    let analysis = cache
        .cached(&env, &main, &[], &files)
        .expect("only hint data changed; compiler semantic proof remains current");
    let findings = findings(analysis, &main);
    assert!(
        findings
            .iter()
            .any(|finding| finding.message.contains("Wanted"))
    );
    assert!(
        !findings
            .iter()
            .any(|finding| finding.display.contains("Widgets")),
        "stale hint must retire despite equal size and mtime"
    );
    assert_eq!(cache.counts().0, 1);
}

#[test]
fn failed_hint_guard_keeps_semantic_queries_and_nonempty_references_available() {
    let (_temp, env, main, module, files) = fixture();
    let mut session = crate::Session::with_environment(crate::Limits::default(), env);
    session.set_auto_import(false);
    let uri = crate::DocumentUri::from_path(&main).unwrap();
    let text = String::from_utf8(files[&main].to_vec()).unwrap();
    session.open(uri.clone(), 1, text.clone()).unwrap();
    assert!(
        session
            .diagnostics(&uri)
            .unwrap()
            .iter()
            .any(|d| d.message.contains("Widgets"))
    );
    session.defer_compiles(true);
    let before = session.compile_counts();
    assert_eq!(
        session.with_semantic(&uri, &text, |analysis, path| {
            let first = analysis.staged_check_findings(path);
            assert!(
                first
                    .iter()
                    .any(|finding| finding.display.contains("Widgets"))
            );
            std::fs::write(&module, "Absent :: 42;").unwrap();
            Some(42)
        }),
        None
    );
    assert_eq!(
        session.with_semantic(&uri, &text, |_, _| Some(43)),
        Some(43)
    );
    let references = session
        .references(&uri, crate::Position::new(0, 11), true)
        .unwrap();
    assert_eq!(
        references.len(),
        2,
        "unchanged helper parameter remains useful"
    );
    assert_eq!(
        session.compile_counts(),
        before,
        "hint invalidity must not force recompilation"
    );
}

#[test]
fn multiple_hint_batches_union_their_observations_until_query_acceptance() {
    let (_temp, env, main, module, files) = fixture();
    let mut session = crate::Session::with_environment(crate::Limits::default(), env);
    session.set_auto_import(false);
    let uri = crate::DocumentUri::from_path(&main).unwrap();
    let text = String::from_utf8(files[&main].to_vec()).unwrap();
    session.open(uri.clone(), 1, text.clone()).unwrap();
    session.diagnostics(&uri).unwrap();
    assert_eq!(
        session.with_semantic(&uri, &text, |analysis, path| {
            assert!(
                analysis
                    .staged_check_findings(path)
                    .iter()
                    .any(|f| f.display.contains("Widgets"))
            );
            std::fs::write(&module, "Absent :: 42;").unwrap();
            assert!(
                !analysis
                    .staged_check_findings(path)
                    .iter()
                    .any(|f| f.display.contains("Widgets"))
            );
            Some(42)
        }),
        None,
        "a last-batch-only proof would incorrectly accept the changed context"
    );
    assert_eq!(
        session.with_semantic(&uri, &text, |_, _| Some(43)),
        Some(43)
    );
}

#[test]
fn unsaved_hint_overlay_precedence_does_not_require_a_new_semantic_compiler() {
    let (_temp, env, main, module, files) = fixture();
    let mut cache = Cache::default();
    let analysis = cache.analyze(&env, &main, &[], files.clone()).unwrap();
    assert!(
        findings(analysis, &main)
            .iter()
            .any(|f| f.display.contains("Widgets"))
    );
    let mut overlaid = (*files).clone();
    overlaid.insert(module, Arc::from(b"Absent :: 42;".as_slice()));
    let analysis = cache.analyze(&env, &main, &[], Rc::new(overlaid)).unwrap();
    assert!(
        !findings(analysis, &main)
            .iter()
            .any(|f| f.display.contains("Widgets"))
    );
    let analysis = cache.analyze(&env, &main, &[], files).unwrap();
    assert!(
        findings(analysis, &main)
            .iter()
            .any(|f| f.display.contains("Widgets"))
    );
    assert_eq!(cache.counts().0, 1);
}

#[test]
fn neighboring_builder_hint_tracks_add_edit_and_removal_without_poisoning_compiler() {
    let (_temp, env, main, _module, files) = fixture();
    let builder = main.parent().unwrap().join("builder.jai");
    let mut cache = Cache::default();
    let analysis = cache.analyze(&env, &main, &[], files.clone()).unwrap();
    assert!(
        !findings(analysis, &main)
            .iter()
            .any(|f| f.display.contains("builder.jai"))
    );
    std::fs::write(&builder, r#"#run add_build_string("Wanted :: 42;");"#).unwrap();
    let analysis = cache
        .cached(&env, &main, &[], &files)
        .expect("unloaded neighbor is only a hint input");
    assert!(
        findings(analysis, &main)
            .iter()
            .any(|f| f.display.contains("builder.jai"))
    );
    std::fs::write(&builder, r#"#run add_build_string("Absent :: 42;");"#).unwrap();
    let analysis = cache.cached(&env, &main, &[], &files).unwrap();
    assert!(
        !findings(analysis, &main)
            .iter()
            .any(|f| f.display.contains("builder.jai"))
    );
    std::fs::remove_file(&builder).unwrap();
    let analysis = cache.cached(&env, &main, &[], &files).unwrap();
    assert!(
        findings(analysis, &main)
            .iter()
            .any(|f| f.display.contains("Widgets"))
    );
    assert_eq!(cache.counts().0, 1);
}

#[test]
fn fresh_hint_context_cannot_consume_a_poisoned_stamp_only_shared_index() {
    let (_temp, env, main, module, files) = fixture();
    let options = (env.options)(&*env.fs, &main);
    let mut old = Compiler::new(options, env.fs.clone());
    let error = old.compile_program(&main).unwrap_err();
    let old_hint = old.with_name_suggestion(&error).unwrap();
    assert!(
        old_hint
            .suggestions()
            .iter()
            .any(|s| s.title.contains("Widgets"))
    );
    let modified = std::fs::metadata(&module).unwrap().modified().unwrap();
    std::fs::write(&module, "Absent :: 42;").unwrap();
    std::fs::File::options()
        .write(true)
        .open(&module)
        .unwrap()
        .set_times(std::fs::FileTimes::new().set_modified(modified))
        .unwrap();
    let mut cache = Cache::default();
    let analysis = cache.analyze(&env, &main, &[], files).unwrap();
    assert!(
        !findings(analysis, &main)
            .iter()
            .any(|f| f.display.contains("Widgets")),
        "a new exact hint batch must own new line scans and parsed cells despite equal stamps"
    );
}

#[test]
fn hint_scope_drops_its_proof_on_panic_and_outermost_success() {
    let (_temp, env, main, module, files) = fixture();
    let mut cache = Cache::default();
    let analysis = cache.analyze(&env, &main, &[], files).unwrap();
    let panic = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let _query = analysis.hint_query();
        analysis.staged_check_findings(&main);
        panic!("test abort after hint capture");
    }));
    assert!(panic.is_err());
    std::fs::write(&module, "Absent :: 42;").unwrap();
    assert!(
        analysis.hints_current(&*env.fs),
        "unwound query retains no old proof"
    );
    let _ = findings(analysis, &main);
    std::fs::write(&module, "Wanted :: 42;").unwrap();
    assert!(
        analysis.hints_current(&*env.fs),
        "successful outer query retains no old proof"
    );
}

#[test]
fn semantic_and_hint_first_observations_must_agree_before_acceptance() {
    struct SequencedFs {
        path: PathBuf,
        reads: std::cell::Cell<usize>,
    }
    impl FileSystem for SequencedFs {
        fn is_native(&self) -> bool {
            true
        }
        fn read(&self, path: &std::path::Path) -> Option<Vec<u8>> {
            if path == self.path {
                let n = self.reads.get();
                self.reads.set(n + 1);
                return Some(if n == 0 {
                    b"semantic".to_vec()
                } else {
                    b"hint".to_vec()
                });
            }
            jaic::sema::NativeFs.read(path)
        }
        fn is_file(&self, path: &std::path::Path) -> bool {
            jaic::sema::NativeFs.is_file(path)
        }
        fn is_dir(&self, path: &std::path::Path) -> bool {
            jaic::sema::NativeFs.is_dir(path)
        }
        fn canonical(&self, path: &std::path::Path) -> PathBuf {
            jaic::sema::NativeFs.canonical(path)
        }
        fn canonical_exact(&self, path: &std::path::Path) -> Option<PathBuf> {
            jaic::sema::NativeFs.canonical_exact(path)
        }
        fn list_dir(&self, path: &std::path::Path) -> Vec<(String, bool)> {
            jaic::sema::NativeFs.list_dir(path)
        }
        fn stamp(&self, path: &std::path::Path) -> Option<u64> {
            jaic::sema::NativeFs.stamp(path)
        }
    }
    let (temp, env, main, _module, files) = fixture();
    let mut cache = Cache::default();
    let analysis = cache.analyze(&env, &main, &[], files).unwrap();
    let path = temp.path().join("shared-response.bin");
    let semantic = super::observations::Observation::Read(Some(Arc::from(b"semantic".as_slice())));
    let hint = super::observations::Observation::Read(Some(Arc::from(b"hint".as_slice())));
    analysis
        .reads
        .borrow_mut()
        .disk
        .record(path.clone(), semantic);
    let mut hints = Reads::default();
    hints.disk.record(path.clone(), hint);
    let fs = SequencedFs {
        path,
        reads: std::cell::Cell::new(0),
    };
    assert!(analysis.reads.borrow().is_current(&fs));
    assert!(
        hints.is_current(&fs),
        "each separate replay can match its own first response"
    );
    fs.reads.set(0);
    let _query = analysis.hint_query();
    analysis.hints.add(analysis, &hints);
    assert!(analysis.reads.borrow().is_current(&fs));
    assert!(
        !analysis.hints_current(&fs),
        "combined output cannot accept contradictory first observations"
    );
}

struct ChurningFs {
    module: PathBuf,
    armed: std::cell::Cell<bool>,
    reads: std::cell::Cell<usize>,
}

impl FileSystem for ChurningFs {
    fn is_native(&self) -> bool {
        true
    }

    fn read(&self, path: &std::path::Path) -> Option<Vec<u8>> {
        if path == self.module && self.armed.get() {
            let n = self.reads.get();
            self.reads.set(n + 1);
            if n == 1 {
                std::fs::write(path, "Absent :: 42;").unwrap();
            }
        }
        jaic::sema::NativeFs.read(path)
    }

    fn is_file(&self, path: &std::path::Path) -> bool {
        jaic::sema::NativeFs.is_file(path)
    }

    fn is_dir(&self, path: &std::path::Path) -> bool {
        jaic::sema::NativeFs.is_dir(path)
    }

    fn canonical(&self, path: &std::path::Path) -> PathBuf {
        jaic::sema::NativeFs.canonical(path)
    }

    fn canonical_exact(&self, path: &std::path::Path) -> Option<PathBuf> {
        jaic::sema::NativeFs.canonical_exact(path)
    }

    fn list_dir(&self, path: &std::path::Path) -> Vec<(String, bool)> {
        jaic::sema::NativeFs.list_dir(path)
    }

    fn stamp(&self, path: &std::path::Path) -> Option<u64> {
        jaic::sema::NativeFs.stamp(path)
    }
}


#[test]
fn changed_between_hint_line_scan_and_lazy_parse_is_unavailable_without_retry() {
    let (_temp, mut env, main, module, files) = fixture();
    let fs = Rc::new(ChurningFs {
        module,
        armed: std::cell::Cell::new(false),
        reads: std::cell::Cell::new(0),
    });
    env.fs = fs.clone();
    let mut session = crate::Session::with_environment(crate::Limits::default(), env);
    session.set_auto_import(false);
    let uri = crate::DocumentUri::from_path(&main).unwrap();
    let text = String::from_utf8(files[&main].to_vec()).unwrap();
    session.open(uri.clone(), 1, text.clone()).unwrap();
    assert_eq!(
        session.with_semantic(&uri, &text, |analysis, path| {
            fs.armed.set(true);
            let _ = analysis.staged_check_findings(path);
            Some(42)
        }),
        None
    );
    assert!(
        fs.reads.get() >= 2,
        "line scan and later read actually saw different responses"
    );
    let compiled = session.compile_counts();
    session.defer_compiles(true);
    assert_eq!(
        session.with_semantic(&uri, &text, |_, _| Some(43)),
        Some(43)
    );
    assert_eq!(
        session.compile_counts(),
        compiled,
        "the semantic compiler remains current"
    );
}

#[test]
fn rejected_hint_output_cannot_clear_an_existing_semantic_error() {
    let (_temp, mut env, main, module, files) = fixture();
    let fs = Rc::new(ChurningFs {
        module,
        armed: std::cell::Cell::new(false),
        reads: std::cell::Cell::new(0),
    });
    env.fs = fs.clone();
    let mut session = crate::Session::with_environment(crate::Limits::default(), env);
    session.set_auto_import(false);
    let uri = crate::DocumentUri::from_path(&main).unwrap();
    let text = String::from_utf8(files[&main].to_vec()).unwrap();
    session.open(uri.clone(), 1, text.clone()).unwrap();
    let prior = session.diagnostics(&uri).unwrap();
    assert!(prior.iter().any(|d| d.message.contains("Wanted")));
    let compiles = session.compile_counts();
    fs.reads.set(0);
    fs.armed.set(true);
    let next = session.diagnostics(&uri);
    assert!(
        next.is_err(),
        "unavailable semantic diagnostics cannot become a clean report: {next:?}"
    );
    assert!(
        fs.reads.get() >= 2,
        "the diagnostic proof actually became inconsistent"
    );
    assert_eq!(session.compile_counts(), compiles);
    assert_eq!(
        session.with_semantic(&uri, &text, |_, _| Some(42)),
        Some(42)
    );
}

#[test]
fn rejected_hint_output_cannot_publish_an_empty_replacement() {
    let (_temp, mut env, main, module, files) = fixture();
    let fs = Rc::new(ChurningFs {
        module,
        armed: std::cell::Cell::new(false),
        reads: std::cell::Cell::new(0),
    });
    env.fs = fs.clone();
    let mut session = crate::Session::with_environment(crate::Limits::default(), env);
    session.set_auto_import(false);
    let uri = crate::DocumentUri::from_path(&main).unwrap();
    let text = String::from_utf8(files[&main].to_vec()).unwrap();
    session.open(uri.clone(), 1, text).unwrap();
    assert!(
        session
            .publications()
            .iter()
            .any(|(_, _, items)| items.iter().any(|d| d.message.contains("Wanted")))
    );
    let compiles = session.compile_counts();
    fs.reads.set(0);
    fs.armed.set(true);
    let next = session.publications();
    assert!(
        next.iter().all(|(document, _, _)| document != uri.as_str()),
        "do not clear accepted diagnostics when replacement is unavailable: {next:?}"
    );
    assert!(fs.reads.get() >= 2);
    assert_eq!(session.compile_counts(), compiles);
}

#[test]
fn raw_operator_hints_cannot_import_stamp_only_index_facts_from_another_compiler() {
    let temp = test_support::TempDir::new("raw-operator-hint-proof");
    let modules = temp.path().join("modules");
    let project = temp.path().join("project/src");
    for dir in [modules.join("Other"), modules.join("Ops"), project.clone()] {
        std::fs::create_dir_all(dir).unwrap();
    }
    std::fs::write(
        modules.join("Other/module.jai"),
        "Thing :: struct { n: int; }",
    )
    .unwrap();
    let ops = modules.join("Ops/module.jai");
    let old_source = "O :: #import \"Other\";\noperator + :: (a: O.Thing, b: O.Thing) -> O.Thing { return .{a.n+b.n}; }";
    std::fs::write(&ops, old_source).unwrap();
    let main = project.join("main.jai");
    let source = "O :: #import \"Other\"; main :: () { a,b: O.Thing; result := a+b; }";
    std::fs::write(&main, source).unwrap();
    let env = Environment {
        fs: Rc::new(jaic::sema::NativeFs),
        services: None,
        options: Box::new(move |_, _| {
            let mut options = Options::host();
            options.runtime_support = false;
            options.stack_trace = false;
            options.preload_stdlib = Some(modules.clone());
            options.import_paths = vec![modules.clone()];
            options
        }),
    };
    let options = (env.options)(&*env.fs, &main);
    // A legacy compiler supplies the same overlay stamp semantics as the LSP. A plain
    // NativeFs producer would have different directory fingerprints and miss the cache.
    let old_fs = Rc::new(super::overlay::OverlayFs {
        base: env.fs.clone(),
        dir: main.parent().unwrap().to_path_buf(),
        files: std::cell::RefCell::default(),
        generated: BTreeMap::new(),
        canonical: jaic::paths::CanonCache::default(),
        reads: Rc::default(),
    });
    let positive_options = options.clone();
    let mut old = Compiler::new(options, old_fs);
    let raw_error = old.compile_program(&main).unwrap_err();
    let old_error = old.with_name_suggestion(&raw_error).unwrap();
    assert!(
        old_error
            .suggestions()
            .iter()
            .any(|s| s.title.contains("Ops")),
        "{old_error:?}"
    );
    let positive_env = Environment {
        fs: env.fs.clone(),
        services: None,
        options: Box::new(move |_, _| positive_options.clone()),
    };
    let mut positive = crate::Session::with_environment(crate::Limits::default(), positive_env);
    positive.set_auto_import(false);
    let positive_uri = crate::DocumentUri::from_path(&main).unwrap();
    positive
        .open(positive_uri.clone(), 1, source.into())
        .unwrap();
    assert!(
        positive
            .diagnostics(&positive_uri)
            .unwrap()
            .iter()
            .any(|d| d.message.contains("Ops")),
        "unchanged valid operator hint remains available"
    );
    assert_eq!(
        positive.with_semantic(&positive_uri, source, |analysis, _| Some(
            analysis.compiler.options.hint_index_mode
        )),
        Some(jaic::sema::HintIndexMode::PerCompiler)
    );
    let modified = std::fs::metadata(&ops).unwrap().modified().unwrap();
    let new_source = old_source.replace("operator +", "operator -");
    assert_eq!(old_source.len(), new_source.len());
    std::fs::write(&ops, new_source).unwrap();
    std::fs::File::options()
        .write(true)
        .open(&ops)
        .unwrap()
        .set_times(std::fs::FileTimes::new().set_modified(modified))
        .unwrap();
    let mut session = crate::Session::with_environment(crate::Limits::default(), env);
    session.set_auto_import(false);
    let uri = crate::DocumentUri::from_path(&main).unwrap();
    session.open(uri.clone(), 1, source.into()).unwrap();
    let diagnostics = session.diagnostics(&uri).unwrap();
    assert!(
        diagnostics.iter().any(|d| d.message.contains("operator")),
        "{diagnostics:?}"
    );
    assert!(
        !diagnostics.iter().any(|d| d.message.contains("Ops")),
        "stale raw operator import hint: {diagnostics:?}"
    );
    let at = source.find("a+b").unwrap() as u32;
    let actions = session
        .code_actions(
            &uri,
            crate::Range::new(crate::Position::new(0, at), crate::Position::new(0, at + 3)),
        )
        .unwrap();
    assert!(
        !actions.iter().any(|a| a.title.contains("Ops")),
        "{actions:?}"
    );
}

#[test]
fn exact_hint_index_policy_reaches_metaprogram_children_and_focus_switches() {
    let build = r#"#import "Basic";
#import "Compiler";
#run build();
build :: () {
    set_build_options_dc(.{do_output = false});
    for string.["first", "second"] {
        w := compiler_create_workspace(it);
        options := get_build_options(w);
        options.output_type = .NO_OUTPUT;
        set_build_options(options, w);
        add_build_file(tprint("%.jai", it), w);
    }
}
"#;
    let dir = test_support::lsp::project(
        "hint-policy-children",
        &[
            ("jacket.toml", "[project]\nentry = [\"build.jai\"]\n"),
            ("build.jai", build),
            ("first.jai", "first :: 42;"),
            ("second.jai", "second :: 43;"),
        ],
    );
    let stdlib = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../stdlib")
        .canonicalize()
        .unwrap();
    let mut session = crate::Session::with_environment(
        crate::Limits::default(),
        Environment {
            fs: Rc::new(jaic::sema::NativeFs),
            services: None,
            options: Box::new(move |fs, main| {
                let inputs = jaic::project::ProgramInputs::for_file(fs, main, &stdlib);
                Options::for_program(fs, main, &inputs)
            }),
        },
    );
    session.set_workspace_folders(vec![dir.path().to_path_buf()]);
    let first_path = dir.path().join("first.jai");
    let first = crate::DocumentUri::from_path(&first_path).unwrap();
    let text = std::fs::read_to_string(first_path).unwrap();
    session.open(first.clone(), 1, text.clone()).unwrap();
    let second = dir.path().join("second.jai");
    session.diagnostics(&first).unwrap();
    assert_eq!(
        session.with_semantic(&first, &text, |analysis, path| {
            let modes: Vec<_> = analysis
                .polymorph_compilers()
                .map(|(compiler, _)| compiler.options.hint_index_mode)
                .collect();
            assert!(
                modes.len() >= 3,
                "producer plus two actually created child compilers"
            );
            assert!(
                modes
                    .iter()
                    .all(|mode| *mode == jaic::sema::HintIndexMode::PerCompiler)
            );
            analysis.focus(&second);
            assert!(analysis.file(&second).is_some());
            assert_eq!(
                analysis.compiler.options.hint_index_mode,
                jaic::sema::HintIndexMode::PerCompiler
            );
            analysis.focus(path);
            Some(42)
        }),
        Some(42)
    );
}

fn protocol_messages(protocol: &mut crate::JsonSession, input: Value) -> Vec<Value> {
    let output: Vec<serde_json::Value> = protocol
        .handle_json(&input.to_string())
        .unwrap()
        .into_iter()
        .map(|text| serde_json::from_str(&text).unwrap())
        .collect();
    if input["method"] == "initialize" && output.first().is_some_and(|m| m.get("result").is_some())
    {
        assert!(
            protocol
                .handle_json(r#"{"jsonrpc":"2.0","method":"initialized","params":{}}"#)
                .unwrap()
                .is_empty()
        );
    }
    output
}

fn hint_protocol(
    pull: bool,
) -> (
    test_support::TempDir,
    Rc<ChurningFs>,
    crate::JsonSession,
    crate::DocumentUri,
    Vec<Value>,
) {
    let (temp, mut env, main, module, files) = fixture();
    let fs = Rc::new(ChurningFs {
        module,
        armed: std::cell::Cell::new(false),
        reads: std::cell::Cell::new(0),
    });
    env.fs = fs.clone();
    let uri = crate::DocumentUri::from_path(&main).unwrap();
    let root = crate::DocumentUri::from_path(main.parent().unwrap()).unwrap();
    let text = String::from_utf8(files[&main].to_vec()).unwrap();
    let mut protocol = crate::JsonSession::with_environment(crate::Limits::default(), env);
    let capabilities = if pull {
        json!({"textDocument":{"diagnostic":{}}})
    } else {
        json!({})
    };
    protocol_messages(
        &mut protocol,
        json!({"jsonrpc":"2.0","id":0,"method":"initialize","params":{"capabilities":capabilities,"rootUri":root.as_str(),"initializationOptions":{"completion":{"autoImport":false}}}}),
    );
    protocol_messages(
        &mut protocol,
        json!({"jsonrpc":"2.0","method":"initialized","params":{}}),
    );
    let opened = protocol_messages(
        &mut protocol,
        json!({"jsonrpc":"2.0","method":"textDocument/didOpen","params":{"textDocument":{"uri":uri.as_str(),"version":1,"languageId":"jai","text":text}}}),
    );
    (temp, fs, protocol, uri, opened)
}

fn diagnostic_pull_rejection(method: &str) {
    let (_temp, fs, mut protocol, uri, _) = hint_protocol(true);
    let prior = protocol_messages(&mut protocol, json!({"jsonrpc":"2.0","id":1,"method":"textDocument/diagnostic","params":{"textDocument":{"uri":uri.as_str()}}})).into_iter().find(|v|v["id"]==1).unwrap();
    assert!(
        prior["result"]["items"]
            .as_array()
            .unwrap()
            .iter()
            .any(|d| d["message"].as_str().unwrap().contains("Wanted"))
    );
    let compiles = protocol.compile_counts();
    fs.reads.set(0);
    fs.armed.set(true);
    let params = if method == "workspace/diagnostic" {
        json!({"previousResultIds":[{"uri":uri.as_str(),"value":prior["result"]["resultId"]}]})
    } else {
        json!({"textDocument":{"uri":uri.as_str()},"previousResultId":prior["result"]["resultId"]})
    };
    let next = protocol_messages(
        &mut protocol,
        json!({"jsonrpc":"2.0","id":2,"method":method,"params":params}),
    )
    .into_iter()
    .find(|v| v["id"] == 2)
    .unwrap();
    assert_eq!(
        next["error"]["code"],
        lsp_types::error_codes::SERVER_CANCELLED,
        "{next}"
    );
    assert!(next.get("result").is_none());
    let data: lsp_types::DiagnosticServerCancellationData =
        serde_json::from_value(next["error"]["data"].clone()).unwrap();
    assert!(
        !data.retrigger_request,
        "wait for a new client trigger instead of retrying unstable evidence"
    );
    assert!(fs.reads.get() >= 2);
    assert_eq!(protocol.compile_counts(), compiles);
    let hover = protocol_messages(&mut protocol,json!({"jsonrpc":"2.0","id":3,"method":"textDocument/hover","params":{"textDocument":{"uri":uri.as_str()},"position":{"line":0,"character":11}}})).into_iter().find(|v|v["id"]==3).unwrap();
    assert!(
        hover.get("error").is_none() && hover["result"].is_object(),
        "{hover}"
    );
    assert_eq!(protocol.compile_counts(), compiles);
}

#[test]
fn rejected_hint_document_pull_preserves_prior_diagnostics() {
    diagnostic_pull_rejection("textDocument/diagnostic");
}

#[test]
fn rejected_hint_workspace_pull_preserves_prior_diagnostics() {
    diagnostic_pull_rejection("workspace/diagnostic");
}

#[test]
fn rejected_hint_push_does_not_emit_a_clean_wire_replacement() {
    let (_temp, fs, mut protocol, uri, opened) = hint_protocol(false);
    assert!(
        opened
            .iter()
            .any(|m| m["method"] == "textDocument/publishDiagnostics"
                && m["params"]["diagnostics"]
                    .as_array()
                    .is_some_and(|items| items
                        .iter()
                        .any(|d| d["message"].as_str().unwrap().contains("Wanted"))))
    );
    fs.reads.set(0);
    fs.armed.set(true);
    let next = protocol_messages(
        &mut protocol,
        json!({"jsonrpc":"2.0","method":"textDocument/didChange","params":{"textDocument":{"uri":uri.as_str(),"version":2},"contentChanges":[]}}),
    );
    assert!(
        next.iter()
            .all(|m| m["method"] != "textDocument/publishDiagnostics"
                || m["params"]["uri"] != uri.as_str()),
        "{next:?}"
    );
    assert!(fs.reads.get() >= 2);
}

#[test]
fn native_parser_diagnostics_and_explicit_syntax_only_sessions_remain_available() {
    let (_temp, env, main, _, _) = fixture();
    let uri = crate::DocumentUri::from_path(&main).unwrap();
    for mut session in [
        crate::Session::with_environment(crate::Limits::default(), env),
        crate::Session::new(crate::Limits::default()),
    ] {
        session.open(uri.clone(), 1, "main :: () {".into()).unwrap();
        let diagnostics = session.diagnostics(&uri).unwrap();
        assert!(
            diagnostics
                .iter()
                .any(|d| d.code == crate::DiagnosticCode::Parser),
            "{diagnostics:?}"
        );
        assert!(session.publications().iter().any(|(_, _, items)| {
            items
                .iter()
                .any(|d| d.code == crate::DiagnosticCode::Parser)
        }));
    }
}
