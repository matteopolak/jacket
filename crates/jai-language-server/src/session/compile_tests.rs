use super::*;

use jaic::sema::{FileSystem, NativeFs};

struct CountingFs {
    path: PathBuf,
    reads: std::cell::Cell<usize>,
}

impl FileSystem for CountingFs {
    fn read(&self, path: &Path) -> Option<Vec<u8>> {
        if path == self.path {
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

#[test]
fn accepted_epoch_discovery_memo_is_coherent_and_new_capture_rejects_churn() {
    let temp = test_support::TempDir::new("discovery-validation-scope");
    let root = temp.path().join("workspace/project/source");
    std::fs::create_dir_all(&root).unwrap();
    let config = root.join("jacket.toml");
    std::fs::write(&config, "[project]\nentry = [\"main.jai\"]\n").unwrap();
    let main = root.join("main.jai");
    let source = "main :: () {}";
    std::fs::write(&main, source).unwrap();
    let fs = Rc::new(CountingFs {
        path: test_support::lsp::native_identity(&config),
        reads: std::cell::Cell::new(0),
    });
    let mut session = Session::with_environment(
        crate::Limits::default(),
        crate::Environment {
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
    let uri = DocumentUri::from_path(&main).unwrap();
    session.open(uri.clone(), 1, source.into()).unwrap();
    session.diagnostics(&uri).unwrap();
    assert!(
        fs.reads.get() > 0,
        "native configuration was actually observed"
    );
    fs.reads.set(0);
    let text = session.document_text(&uri).unwrap().to_owned();
    assert_eq!(
        session.with_semantic(&uri, &text, |_, _| Some(42)),
        Some(42)
    );
    assert_eq!(
        fs.reads.get(),
        0,
        "an accepted epoch does not replay native discovery inputs on each query"
    );
    {
        let _scope = session.discovery_query();
        std::fs::write(&config, "[project]\nentry = [\"other.jai\"]\n").unwrap();
        assert_eq!(
            session.with_semantic(&uri, &text, |_, _| Some(42)),
            Some(42),
            "silent changes do not replace the accepted epoch before an event or reconciliation"
        );
    }
    std::fs::write(&config, "[project]\nentry = [\"main.jai\"]\n").unwrap();
    session.file_changed(&config, false);
    assert_eq!(
        session.with_semantic(&uri, &text, |_, _| {
            std::fs::write(&config, "[project]\nentry = [\"other.jai\"]\n").unwrap();
            Some(42)
        }),
        None,
        "new-epoch acceptance still rejects a config change during the callback"
    );
}

#[test]
fn exact_query_compares_epoch_proof_once_before_and_once_after() {
    let temp = test_support::TempDir::new("query-validation-grant");
    let root = temp.path().join("workspace/project/source");
    std::fs::create_dir_all(&root).unwrap();
    let data = root.join("data.txt");
    std::fs::write(&data, "observed").unwrap();
    let fs = Rc::new(CountingFs {
        path: data.clone(),
        reads: std::cell::Cell::new(0),
    });
    let mut session = Session::with_environment(
        crate::Limits::default(),
        crate::Environment {
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
    let uri = DocumentUri::from_path(&root.join("main.jai")).unwrap();
    let source = "main :: () {}";
    session.open(uri.clone(), 1, source.into()).unwrap();
    session
        .with_semantic(&uri, source, |analysis, _| {
            analysis.compiler.src.fs.read(&data)
        })
        .unwrap();
    fs.reads.set(0);
    let logical = crate::reference_records::request_work_tests::ReadComparisons::start(&fs.path);
    assert_eq!(
        session.with_semantic(&uri, source, |_, _| Some(42)),
        Some(42)
    );
    assert_eq!(
        logical.count(),
        2,
        "the original logical proof comparison count is unchanged"
    );
    assert_eq!(
        fs.reads.get(),
        0,
        "warm epoch proof comparisons do not read native bytes"
    );
    drop(logical);
    session.defer_compiles(true);
    let edited = format!("{source}// typing");
    session
        .change(
            &uri,
            2,
            &[crate::TextChange {
                range: None,
                range_length: None,
                text: edited.clone(),
            }],
        )
        .unwrap();
    fs.reads.set(0);
    let logical = crate::reference_records::request_work_tests::ReadComparisons::start(&fs.path);
    assert_eq!(
        session.on_older_compile(&uri, &edited, |_, _, _| Some(42)),
        Some(42)
    );
    assert_eq!(
        fs.reads.get(),
        0,
        "shifted grant reads the accepted epoch, with no native replay"
    );
    assert_eq!(
        logical.count(),
        2,
        "shifted proof still has one pre/post comparison"
    );
    drop(logical);
    fs.reads.set(0);
    let logical = crate::reference_records::request_work_tests::ReadComparisons::start(&fs.path);
    assert_eq!(
        session.with_compiled_before(&uri, &edited, 8, |_, _| Some(42)),
        Some(42)
    );
    assert_eq!(
        fs.reads.get(),
        0,
        "prefix grant reads the accepted epoch, with no native replay"
    );
    assert_eq!(
        logical.count(),
        2,
        "prefix proof still has one pre/post comparison"
    );
    drop(logical);
    session
        .change(
            &uri,
            3,
            &[crate::TextChange {
                range: None,
                range_length: None,
                text: source.into(),
            }],
        )
        .unwrap();
    session.defer_compiles(false);
    // Native callback churn is an admission test, not warm-epoch silent freshness.
    session.input_epoch.as_ref().unwrap().begin_capture();
    let rejected = session.with_semantic(&uri, source, |_, _| {
        std::fs::write(&data, "changed").unwrap();
        Some(42)
    });
    assert_eq!(
        rejected, None,
        "new-capture final native validation remains mandatory"
    );
}

fn counted_semantic_dependency(
    source: &str,
) -> (test_support::TempDir, Rc<CountingFs>, Session, DocumentUri) {
    let temp = test_support::TempDir::new("component-validation");
    let root = temp.path().join("workspace/project/source");
    std::fs::create_dir_all(&root).unwrap();
    let data = root.join("data.txt");
    std::fs::write(&data, "observed").unwrap();
    let fs = Rc::new(CountingFs {
        path: data,
        reads: std::cell::Cell::new(0),
    });
    let mut session = Session::with_environment(
        crate::Limits::default(),
        crate::Environment {
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
    let uri = DocumentUri::from_path(&root.join("main.jai")).unwrap();
    session.open(uri.clone(), 1, source.into()).unwrap();
    session
        .with_semantic(&uri, source, |a, _| a.compiler.src.fs.read(&fs.path))
        .unwrap();
    fs.reads.set(0);
    (temp, fs, session, uri)
}

#[test]
fn diagnostic_components_share_one_checked_context() {
    let (_temp, fs, session, uri) = counted_semantic_dependency("main :: () {}");
    let logical = crate::reference_records::request_work_tests::ReadComparisons::start(&fs.path);
    let before = session.compile_counts();
    assert!(session.diagnostics(&uri).unwrap().is_empty());
    assert_eq!(
        logical.count(),
        2,
        "checks, reachability and lints need one selected context and final replay"
    );
    assert_eq!(
        fs.reads.get(),
        0,
        "an accepted warm epoch makes no native reads"
    );
    assert_eq!(session.compile_counts(), before);
}

#[test]
fn absent_incomplete_switch_does_not_rebuild_diagnostic_hints() {
    let (_temp, fs, session, uri) = counted_semantic_dependency("main :: () {}");
    let logical = crate::reference_records::request_work_tests::ReadComparisons::start(&fs.path);
    let actions =
        session.complete_switch_actions(&uri, 0, 0, &crate::lints::ActionContext::default());
    assert!(actions.is_empty());
    assert_eq!(
        logical.count(),
        2,
        "an accepted empty switch-error set cannot produce a switch quick fix"
    );
    assert_eq!(
        fs.reads.get(),
        0,
        "an accepted warm epoch makes no native reads"
    );
}

#[test]
fn format_hover_outside_literal_never_selects_semantics() {
    let (_temp, fs, session, uri) =
        counted_semantic_dependency("consume :: (value: string) {} main :: () { consume(\"%\"); }");
    assert_eq!(session.analysis_for(&uri).unwrap().format_calls.len(), 1);
    assert!(session.format_hover(&uri, 0).is_none());
    assert_eq!(
        fs.reads.get(),
        0,
        "cursor admission uses the existing syntax candidate spans before semantic callee lookup"
    );
}

#[test]
fn alternate_query_validates_discovery_even_without_reference_publication() {
    use jaic::sema::FileSystem;
    let temp = test_support::TempDir::new("alternate-discovery-guard");
    let root = temp.path().join("workspace/project/source");
    std::fs::create_dir_all(&root).unwrap();
    let main = root.join("main.jai");
    let data = root.join("config-data.txt");
    std::fs::write(&data, "before").unwrap();
    let session = Session::with_environment(
        crate::Limits::default(),
        crate::Environment {
            fs: Rc::new(jaic::sema::NativeFs),
            services: None,
            options: Box::new(|_, _| {
                let mut options = jaic::sema::Options::host();
                options.runtime_support = false;
                options.stack_trace = false;
                options
            }),
        },
    );
    let files = Rc::new(BTreeMap::from([(
        main.clone(),
        Arc::<[u8]>::from(&b"main :: () {}"[..]),
    )]));
    let mut cache = semantic::Cache::default();
    let analysis = cache
        .analyze_for(
            session.environment.as_ref().unwrap(),
            &main,
            &[],
            files,
            Some(jaic::target::TargetOs::Linux),
        )
        .unwrap();
    assert!(!analysis.reference_records_allowed());
    let analysis = QueryAnalysis::validate(
        analysis,
        session.environment.as_ref().unwrap(),
        &session.input_proof.borrow(),
    )
    .unwrap();
    let answer = session.semantic_query(analysis, &main, |_, _| {
        assert_eq!(
            session.config_filesystem().unwrap().read(&data).unwrap(),
            b"before"
        );
        std::fs::write(&data, "after").unwrap();
        Some(42)
    });
    assert_eq!(
        answer, None,
        "discovery guard is independent of reference eligibility"
    );
}

#[test]
fn silent_conditional_dependency_retires_alternate_target_negative_spans() {
    let temp = test_support::TempDir::new("alternate-proof");
    let root = temp.path().join("workspace/project/source");
    std::fs::create_dir_all(&root).unwrap();
    let flag = root.join("flag.jai");
    std::fs::write(&flag, "ENABLED :: false;").unwrap();
    let source = "#import \"Basic\"; #load \"flag.jai\"; #if OS == .MACOS && ENABLED { selected :: 42; main :: () { value := selected; } }";
    let uri = crate::DocumentUri::from_path(&root.join("main.jai")).unwrap();
    let env = test_support::lsp::modules_environment(Some(jaic::target::TargetOs::Linux));
    let mut session = Session::with_environment(
        crate::Limits::default(),
        crate::Environment {
            fs: env.fs,
            options: env.options,
            services: env.services,
        },
    );
    session.open(uri.clone(), 1, source.into()).unwrap();
    let byte = source.rfind("selected").unwrap() + 1;
    let query = |analysis: &mut crate::semantic::Analysis, path: &std::path::Path, at| {
        analysis.hover(path, at)
    };
    assert!(
        session
            .with_semantic_at(&uri, source, byte, query)
            .is_none()
    );
    assert!(
        !session.unreached.borrow().1.is_empty(),
        "must exercise retained alternate negatives"
    );
    std::fs::write(&flag, "ENABLED :: true;").unwrap();
    assert!(
        session
            .with_semantic_at(&uri, source, byte, query)
            .is_none(),
        "silent changes retain accepted alternate-target negatives until reconciliation or an event"
    );
    session.file_changed(&flag, false);
    assert!(
        session
            .with_semantic_at(&uri, source, byte, query)
            .is_some(),
        "closed dependency can make a formerly skipped target reachable"
    );
}

#[test]
fn configured_entry_selection_observes_its_actual_file_predicate() {
    struct BlinkFs {
        entry: PathBuf,
        calls: std::cell::Cell<usize>,
        absent_at: Option<usize>,
        changed: std::cell::Cell<bool>,
    }
    impl FileSystem for BlinkFs {
        fn is_native(&self) -> bool {
            true
        }
        fn read(&self, path: &Path) -> Option<Vec<u8>> {
            NativeFs.read(path)
        }
        fn is_file(&self, path: &Path) -> bool {
            if path == self.entry {
                let call = self.calls.get() + 1;
                self.calls.set(call);
                if self.absent_at == Some(call) {
                    let parked = self.entry.with_extension("parked");
                    std::fs::rename(&self.entry, &parked).unwrap();
                    let result = NativeFs.is_file(path);
                    std::fs::rename(&parked, &self.entry).unwrap();
                    self.changed.set(true);
                    assert!(!result);
                    return result;
                }
            }
            NativeFs.is_file(path)
        }
        fn is_dir(&self, path: &Path) -> bool {
            NativeFs.is_dir(path)
        }
        fn list_dir(&self, path: &Path) -> Vec<(String, bool)> {
            NativeFs.list_dir(path)
        }
        fn canonical(&self, path: &Path) -> PathBuf {
            NativeFs.canonical(path)
        }
        fn canonical_exact(&self, path: &Path) -> Option<PathBuf> {
            NativeFs.canonical_exact(path)
        }
        fn stamp(&self, path: &Path) -> Option<u64> {
            NativeFs.stamp(path)
        }
    }
    let trial = |absent_at| {
        let dir = test_support::lsp::project(
            "entry-file-predicate",
            &[
                ("jacket.toml", "[project]\nentry=[\"entry.jai\"]\n"),
                ("entry.jai", "#load \"child.jai\";"),
                ("child.jai", "main :: () {}"),
            ],
        );
        let fs = Rc::new(BlinkFs {
            entry: test_support::lsp::native_identity(&dir.join("entry.jai")),
            calls: std::cell::Cell::new(0),
            absent_at,
            changed: std::cell::Cell::new(false),
        });
        let supplied = test_support::lsp::environment();
        let environment = crate::Environment {
            fs: fs.clone(),
            options: supplied.options,
            services: supplied.services,
        };
        let mut session = Session::with_environment(crate::Limits::default(), environment);
        let uri = DocumentUri::from_path(&dir.join("child.jai")).unwrap();
        let source = "main :: () {}";
        session.open(uri.clone(), 1, source.into()).unwrap();
        let selected = session.compile_inputs(&uri, source);
        let current = session.discovery_inputs_current();
        assert_eq!(
            session.compile_counts().0,
            0,
            "ordinary loader ownership needs no producer compile"
        );
        let is_entry = selected
            .as_ref()
            .is_some_and(|(_, root, _)| root == &fs.entry);
        let is_child = selected.as_ref().is_some_and(|(_, root, _)| {
            root == &test_support::lsp::native_identity(&dir.join("child.jai"))
        });
        (
            is_entry,
            is_child,
            current,
            fs.calls.get(),
            fs.changed.get(),
        )
    };
    let stable = trial(None);
    assert!(
        stable.0 && stable.2,
        "stable configured-entry control: {stable:?}"
    );
    assert!(stable.3 > 0);
    for call in 1..=stable.3 {
        let found = trial(Some(call));
        assert!(
            !(found.1 && found.2),
            "a transient actual false response selected the child while its discovery proof blessed only restored entry responses: call={call}, result={found:?}"
        );
    }
}

#[test]
fn ordinary_hover_gathers_macro_facts_in_its_selected_query() {
    let dir = test_support::lsp::project(
        "hover-selected-query",
        &[
            (
                "main.jai",
                "#load \"dependency.jai\"; main :: () { local := supplied(); local += 1; consume(local); }",
            ),
            (
                "dependency.jai",
                "supplied :: () -> int { return 1; } consume :: (value: int) {}",
            ),
        ],
    );
    let fs = Rc::new(CountingFs {
        path: test_support::lsp::native_identity(&dir.join("dependency.jai")),
        reads: std::cell::Cell::new(0),
    });
    let supplied = test_support::lsp::environment();
    let environment = crate::Environment {
        fs: fs.clone(),
        options: supplied.options,
        services: supplied.services,
    };
    let mut session = Session::with_environment(crate::Limits::default(), environment);
    let uri = DocumentUri::from_path(&dir.join("main.jai")).unwrap();
    let source = std::fs::read_to_string(dir.join("main.jai")).unwrap();
    session.open(uri.clone(), 1, source.clone()).unwrap();
    session.diagnostics(&uri).unwrap();
    let position = test_support::lsp::at_nth(&source, "local", 0, 1);
    let expected = session.hover(&uri, position).unwrap().unwrap();
    assert!(expected.contents.value.contains("local"));
    fs.reads.set(0);
    let logical = crate::reference_records::request_work_tests::ReadComparisons::start(&fs.path);
    assert_eq!(session.hover(&uri, position).unwrap(), Some(expected));
    assert_eq!(
        logical.count(),
        3,
        "one discovery validation, one selected preguard and one final guard; macro enrichment must not select another compiler"
    );
    assert_eq!(
        fs.reads.get(),
        0,
        "an accepted warm epoch makes no native reads"
    );
}

#[test]
fn macro_facts_are_stable_across_normal_hover_refresh() {
    macro_refresh(0);
}

#[test]
fn macro_facts_are_stable_across_partial_hover_refresh() {
    macro_refresh(1);
}

#[test]
fn macro_facts_are_stable_across_focused_child_hover_refresh() {
    macro_refresh(2);
}

fn macro_refresh(mode: usize) {
    let mut source = "square :: (value: int) -> int #expand { return value * value; }\nconsume :: (value: int) {}\nmain :: () { local := square(3); consume(local); }\n".to_owned();
    if mode == 1 {
        source.push_str("broken :: (value: Missing_Refresh_Test_Type) { consume(1); }\n");
    }
    let build = r#"#import "Basic";
#import "Compiler";
#run {
set_build_options_dc(.{do_output=false});
child := compiler_create_workspace("child");
options := get_build_options(child); options.output_type=.NO_OUTPUT; set_build_options(options,child);
add_build_file(tprint("%/main.jai", #filepath), child);
}
"#;
    let mut files = vec![("main.jai", source.as_str())];
    if mode == 2 {
        files.push(("jacket.toml", "[project]\nentry=[\"build.jai\"]\n"));
        files.push(("build.jai", build));
    }
    let dir = test_support::lsp::project("macro-refresh-facts", &files);
    let supplied = test_support::lsp::environment();
    let environment = crate::Environment {
        fs: supplied.fs,
        options: supplied.options,
        services: supplied.services,
    };
    let mut session = Session::with_environment(crate::Limits::default(), environment);
    let uri = DocumentUri::from_path(&dir.join("main.jai")).unwrap();
    session.open(uri.clone(), 1, source.clone()).unwrap();
    session.diagnostics(&uri).unwrap();
    assert!(
        session.call_graphs.borrow().contains_key(&uri),
        "warm diagnostics must establish a graph owner"
    );
    let position = test_support::lsp::at_nth(&source, "square(3)", 0, 1);
    let byte = session
        .document(&uri)
        .unwrap()
        .index
        .byte(&source, position);
    let first = session
        .with_semantic(&uri, &source, |analysis, path| {
            let facts = analysis.hover(path, byte)?;
            assert!(facts.text.contains("square"));
            let file = analysis.file(path)?;
            let expansions = analysis.compiler.ide_expansions(file);
            assert!(!expansions.is_empty(), "useful recorded macro control");
            if mode == 2 {
                assert_ne!(
                    analysis.compiler.meta.workspace,
                    jaic::build::TOP_LEVEL_WORKSPACE,
                    "must focus the actual built child Program"
                );
            }
            Some(format!("{expansions:?}"))
        })
        .unwrap();
    // The first callback's normal refresh and final guard have completed here.
    let after = session
        .with_semantic(&uri, &source, |analysis, path| {
            Some(format!(
                "{:?}",
                analysis.compiler.ide_expansions(analysis.file(path)?)
            ))
        })
        .unwrap();
    assert_eq!(
        first, after,
        "graph refresh must preserve all recorded macro spans/kinds/text/details/output"
    );
}

#[test]
fn unchanged_references_validate_a_published_view_once() {
    let dir = test_support::lsp::project(
        "references-selected-view",
        &[
            (
                "main.jai",
                "#load \"dependency.jai\"; main :: () { local := supplied(); local += 1; consume(local); }",
            ),
            (
                "dependency.jai",
                "supplied :: () -> int { return 1; } consume :: (value: int) {}",
            ),
        ],
    );
    let fs = Rc::new(CountingFs {
        path: test_support::lsp::native_identity(&dir.join("dependency.jai")),
        reads: std::cell::Cell::new(0),
    });
    let supplied = test_support::lsp::environment();
    let environment = crate::Environment {
        fs: fs.clone(),
        options: supplied.options,
        services: supplied.services,
    };
    let mut session = Session::with_environment(crate::Limits::default(), environment);
    let uri = DocumentUri::from_path(&dir.join("main.jai")).unwrap();
    let source = std::fs::read_to_string(dir.join("main.jai")).unwrap();
    session.open(uri.clone(), 1, source.clone()).unwrap();
    session.diagnostics(&uri).unwrap();
    let position = test_support::lsp::at_nth(&source, "local", 0, 1);
    let expected = session.references(&uri, position, true).unwrap();
    assert_eq!(expected.len(), 3);
    fs.reads.set(0);
    let logical = crate::reference_records::request_work_tests::ReadComparisons::start(&fs.path);
    assert_eq!(session.references(&uri, position, true).unwrap(), expected);
    assert_eq!(
        logical.count(),
        2,
        "one discovery validation and one final union replay; an already published exact view needs no mutable compiler callback"
    );
    assert_eq!(
        fs.reads.get(),
        0,
        "an accepted warm epoch makes no native reads"
    );
}

#[test]
fn frozen_execution_lints_use_retained_topology_after_editor_release() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let path = root
        .join("tests/compiler/regressions/run-result-frozen-values.jai")
        .canonicalize()
        .unwrap();
    let text = std::fs::read_to_string(&path).unwrap();
    let environment = test_support::lsp::environment();
    let mut session = Session::with_environment(
        crate::Limits::default(),
        crate::Environment {
            fs: environment.fs,
            services: None,
            options: environment.options,
        },
    );
    let config = DocumentUri::from_path(&path.parent().unwrap().join("jacket.toml")).unwrap();
    session
        .open(
            config,
            1,
            "[project]\nentry=[\"run-result-frozen-values.jai\"]\n".into(),
        )
        .unwrap();
    let uri = DocumentUri::from_path(&path).unwrap();
    session.open(uri.clone(), 1, text.clone()).unwrap();
    let diagnostics = session.diagnostics(&uri).unwrap();
    assert!(
        !diagnostics
            .iter()
            .any(|d| d.code == crate::DiagnosticCode::Lint("returned_local_address")),
        "{diagnostics:?}"
    );
    let usage = session
        .with_semantic(&uri, &text, |a, _| {
            Some(a.compiler.lifetime_execution_usage())
        })
        .unwrap();
    assert!(
        usage.0 > 0,
        "the actual adapter supplies complete-program retention at release"
    );
}

struct ContextCountingFs(std::cell::Cell<usize>);

impl FileSystem for ContextCountingFs {
    fn is_native(&self) -> bool {
        true
    }

    fn read(&self, path: &Path) -> Option<Vec<u8>> {
        self.0.set(self.0.get() + 1);
        NativeFs.read(path)
    }

    fn is_file(&self, path: &Path) -> bool {
        self.0.set(self.0.get() + 1);
        NativeFs.is_file(path)
    }

    fn is_dir(&self, path: &Path) -> bool {
        self.0.set(self.0.get() + 1);
        NativeFs.is_dir(path)
    }

    fn list_dir(&self, path: &Path) -> Vec<(String, bool)> {
        self.0.set(self.0.get() + 1);
        NativeFs.list_dir(path)
    }

    fn canonical(&self, path: &Path) -> PathBuf {
        self.0.set(self.0.get() + 1);
        NativeFs.canonical(path)
    }

    fn canonical_exact(&self, path: &Path) -> Option<PathBuf> {
        self.0.set(self.0.get() + 1);
        NativeFs.canonical_exact(path)
    }

    fn missing_file_key(&self, path: &Path) -> Option<PathBuf> {
        self.0.set(self.0.get() + 1);
        NativeFs.missing_file_key(path)
    }

    fn stamp(&self, path: &Path) -> Option<u64> {
        self.0.set(self.0.get() + 1);
        NativeFs.stamp(path)
    }
}

#[path = "input_epoch_tests.rs"]
mod input_epochs;

#[path = "input_epoch_lifecycle_tests.rs"]
mod input_epoch_lifecycle;
