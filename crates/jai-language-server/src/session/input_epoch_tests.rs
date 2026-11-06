use super::*;

#[test]
fn accepted_input_epoch_keeps_silent_closed_change_until_known_event() {
    let temp = test_support::TempDir::new("input-epoch-known-event");
    let main = temp.path().join("main.jai");
    let dependency = temp.path().join("dependency.jai");
    let source =
        "#load \"dependency.jai\"; main :: () { value: Value; value = 1; if value == 1 {} }";
    std::fs::write(&main, source).unwrap();
    std::fs::write(&dependency, "Value :: int;").unwrap();
    let fs = Rc::new(CountingFs {
        path: test_support::lsp::native_identity(&dependency),
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
    assert!(session.diagnostics(&uri).unwrap().is_empty());
    let accepted = session.compile_counts();
    fs.reads.set(0);
    std::fs::write(&dependency, "Value :: struct { field: int; };").unwrap();
    // Intentional product-policy difference from the historical strict-before-query path:
    // this remains the coherent accepted epoch until an event or reconciliation knows more.
    assert!(session.diagnostics(&uri).unwrap().is_empty());
    assert_eq!(session.compile_counts(), accepted);
    assert_eq!(
        fs.reads.get(),
        0,
        "warm accepted epoch must not replay native bytes"
    );
    session.file_changed(&dependency, false);
    assert!(!session.diagnostics(&uri).unwrap().is_empty());
}

#[test]
fn missed_closed_input_change_is_detected_by_bounded_reconciliation() {
    let temp = test_support::TempDir::new("input-epoch-known-event");
    let main = temp.path().join("main.jai");
    let dependency = temp.path().join("dependency.jai");
    let source =
        "#load \"dependency.jai\"; main :: () { value: Value; value = 1; if value == 1 {} }";
    std::fs::write(&main, source).unwrap();
    std::fs::write(&dependency, "Value :: int;").unwrap();
    let fs = Rc::new(CountingFs {
        path: test_support::lsp::native_identity(&dependency),
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
    assert!(session.diagnostics(&uri).unwrap().is_empty());
    let accepted = session.compile_counts();
    fs.reads.set(0);
    std::fs::write(&dependency, "Value :: struct { field: int; };").unwrap();
    // Intentional product-policy difference from the historical strict-before-query path:
    // this remains the coherent accepted epoch until an event or reconciliation knows more.
    assert!(session.diagnostics(&uri).unwrap().is_empty());
    assert_eq!(session.compile_counts(), accepted);
    assert_eq!(
        fs.reads.get(),
        0,
        "warm accepted epoch must not replay native bytes"
    );
    let changed = (0..256).any(|_| {
        // Continuous useful requests do not reset the reconciliation cursor.
        assert!(session.diagnostics(&uri).unwrap().is_empty());
        session.reconcile_input_epoch()
    });
    assert!(
        changed,
        "a bounded round must discover the missed input change"
    );
    assert!(!session.diagnostics(&uri).unwrap().is_empty());
}

#[test]
fn literal_link_only_session_accepts_and_reconciles_its_input_epoch() {
    let temp = test_support::TempDir::new("input-epoch-literal-only");
    let main = temp.path().join("main.jai");
    let target = temp.path().join("target.jai");
    let source = "#load \"target.jai\";";
    std::fs::write(&main, source).unwrap();
    std::fs::write(&target, "value :: 1;").unwrap();
    let mut session = Session::with_environment(
        crate::Limits::default(),
        crate::Environment {
            fs: Rc::new(NativeFs),
            services: None,
            options: Box::new(|_, _| jaic::sema::Options::host()),
        },
    );
    let uri = DocumentUri::from_path(&main).unwrap();
    session.open(uri.clone(), 1, source.into()).unwrap();
    assert_eq!(session.document_links(&uri).unwrap().len(), 1);
    assert!(
        session.has_input_epoch_work(),
        "literal-only publication must enable reconciliation"
    );
    let counts = session.compile_counts().0;
    std::fs::remove_file(&target).unwrap();
    assert_eq!(
        session.document_links(&uri).unwrap().len(),
        1,
        "silent changes retain accepted links until reconciliation"
    );
    assert!((0..256).any(|_| session.reconcile_input_epoch()));
    assert!(session.document_links(&uri).unwrap().is_empty());
    assert_eq!(
        session.compile_counts().0,
        counts,
        "literal links do not compile a program"
    );
}

#[test]
fn newly_covered_programs_share_a_sealed_successor_without_warm_native_reads() {
    let temp = test_support::TempDir::new("input-epoch-two-programs");
    let a = temp.path().join("a.jai");
    let b = temp.path().join("b.jai");
    let dependency_a = temp.path().join("dependency-a.jai");
    let dependency_b = temp.path().join("dependency-b.jai");
    let source_a =
        "#load \"dependency-a.jai\"; main :: () { value: A; value = 1; if value == 1 {} }";
    let source_b =
        "#load \"dependency-b.jai\"; main :: () { value: B; value = 1; if value == 1 {} }";
    std::fs::write(&a, source_a).unwrap();
    std::fs::write(&b, source_b).unwrap();
    std::fs::write(&dependency_a, "A :: int;").unwrap();
    std::fs::write(&dependency_b, "B :: int;").unwrap();
    let fs = Rc::new(CountingFs {
        path: test_support::lsp::native_identity(&dependency_a),
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
    let uri_a = DocumentUri::from_path(&a).unwrap();
    let uri_b = DocumentUri::from_path(&b).unwrap();
    session.open(uri_a.clone(), 1, source_a.into()).unwrap();
    session.open(uri_b.clone(), 1, source_b.into()).unwrap();
    assert!(session.diagnostics(&uri_a).unwrap().is_empty());
    let mut accepted_b = false;
    for _ in 0..128 {
        match session.diagnostics(&uri_b) {
            Ok(rows) => {
                assert!(rows.is_empty());
                accepted_b = true;
                break;
            }
            Err(crate::Error::DiagnosticsUnavailable) => {}
            Err(error) => panic!("unexpected diagnostic failure: {error:?}"),
        }
        for _ in 0..128 {
            session.reconcile_input_epoch();
            if session.input_epoch.as_ref().unwrap().ready() {
                break;
            }
        }
    }
    assert!(
        accepted_b,
        "finite missing coverage must eventually produce a useful second program"
    );
    assert!(session.diagnostics(&uri_a).unwrap().is_empty());
    let compiles = session.compile_counts().0;
    fs.reads.set(0);
    for _ in 0..8 {
        assert!(session.diagnostics(&uri_a).unwrap().is_empty());
        assert!(session.diagnostics(&uri_b).unwrap().is_empty());
    }
    assert_eq!(session.compile_counts().0, compiles);
    assert_eq!(
        fs.reads.get(),
        0,
        "switching covered programs stays in sealed input authority"
    );
}

#[test]
fn discovery_memo_keeps_actual_option_listing_history_after_compiler_eviction() {
    struct OrderedFs {
        directory: PathBuf,
        reversed: std::cell::Cell<bool>,
    }
    impl FileSystem for OrderedFs {
        fn read(&self, path: &Path) -> Option<Vec<u8>> {
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
        fn missing_file_key(&self, path: &Path) -> Option<PathBuf> {
            NativeFs.missing_file_key(path)
        }
        fn stamp(&self, path: &Path) -> Option<u64> {
            NativeFs.stamp(path)
        }
        fn list_dir(&self, path: &Path) -> Vec<(String, bool)> {
            let mut rows = NativeFs.list_dir(path);
            if path == self.directory {
                rows.sort();
                if self.reversed.get() {
                    rows.reverse();
                }
            }
            rows
        }
        fn is_native(&self) -> bool {
            true
        }
    }
    let temp = test_support::TempDir::new("input-epoch-memo-origin");
    let main = temp.path().join("main.jai");
    let catalog = test_support::lsp::native_identity(&temp.path().join("choices"));
    std::fs::create_dir_all(catalog.join("a")).unwrap();
    std::fs::create_dir_all(catalog.join("b")).unwrap();
    let source = "main :: () {}";
    std::fs::write(&main, source).unwrap();
    let fs = Rc::new(OrderedFs {
        directory: catalog.clone(),
        reversed: std::cell::Cell::new(false),
    });
    let chosen = Rc::new(std::cell::RefCell::new(None));
    let option_chosen = chosen.clone();
    let choices = catalog.clone();
    let mut session = Session::with_environment(
        crate::Limits::default(),
        crate::Environment {
            fs: fs.clone(),
            services: None,
            options: Box::new(move |fs, _| {
                let rows = fs.list_dir(&choices);
                let first = choices.join(&rows.first().expect("two existing choices").0);
                *option_chosen.borrow_mut() = Some(first.clone());
                let mut options = jaic::sema::Options::host();
                options.runtime_support = false;
                options.stack_trace = false;
                options.import_paths = vec![first];
                options
            }),
        },
    );
    let uri = DocumentUri::from_path(&main).unwrap();
    session.open(uri.clone(), 1, source.into()).unwrap();
    assert!(session.diagnostics(&uri).unwrap().is_empty());
    assert_eq!(*chosen.borrow(), Some(catalog.join("a")));
    assert!(session.inputs.borrow().is_some());
    let generation = session.semantic_revision();
    let compiles = session.compile_counts().0;
    assert!(session.compile_inputs(&uri, source).is_some());
    assert_eq!(
        session.inputs.borrow().as_ref().unwrap().2.as_deref(),
        Some(source),
        "the repeated probe-text call must actually select the memo fast path"
    );
    session.semantic.borrow_mut().forget_compiles();
    session
        .input_epoch
        .as_ref()
        .unwrap()
        .begin_request_capture();
    fs.reversed.set(true);
    assert!(
        session.memo_input_proof.borrow().matches(&*fs),
        "normalized listing proof still agrees"
    );
    assert!(
        session.compile_inputs(&uri, source).is_none(),
        "old selection cannot be relabeled with changed actual options input after compiler eviction"
    );
    assert_eq!(session.semantic_revision(), generation);
    assert_eq!(
        session.compile_counts().0,
        compiles,
        "failed memo grant cannot start a same-request compile"
    );
    assert!(session.inputs.borrow().is_none());
    assert!(
        session.compile_inputs(&uri, source).is_some(),
        "a later native transaction can discover fresh inputs"
    );
    assert_eq!(*chosen.borrow(), Some(catalog.join("b")));
    assert_eq!(session.compile_counts().0, compiles);
}

#[test]
fn unchanged_oversized_discovery_memo_owns_its_original_before_native_replay() {
    let temp = test_support::TempDir::new("input-epoch-large-memo-owner");
    let main = temp.path().join("main.jai");
    let data = temp.path().join("data.bin");
    let length = (16 << 20) + 1;
    std::fs::write(&main, "main :: () {}").unwrap();
    std::fs::write(&data, vec![b'x'; length]).unwrap();
    let read = data.clone();
    let mut session = Session::with_environment(
        crate::Limits::default(),
        crate::Environment {
            fs: Rc::new(NativeFs),
            services: None,
            options: Box::new(move |fs, _| {
                assert_eq!(fs.read(&read).unwrap().len(), length);
                let mut options = jaic::sema::Options::host();
                options.runtime_support = false;
                options.stack_trace = false;
                options
            }),
        },
    );
    let uri = DocumentUri::from_path(&main).unwrap();
    session
        .open(uri.clone(), 1, "main :: () {}".into())
        .unwrap();
    assert!(session.diagnostics(&uri).unwrap().is_empty());
    let compiles = session.compile_counts().0;
    assert!(session.compile_inputs(&uri, "main :: () {}").is_some());
    session.semantic.borrow_mut().forget_compiles();
    session
        .input_epoch
        .as_ref()
        .unwrap()
        .begin_request_capture();
    assert!(
        session.compile_inputs(&uri, "main :: () {}").is_some(),
        "unchanged original proof owner must survive native replay without an artificial miss"
    );
    assert!(session.accept_input_epoch());
    assert_eq!(session.compile_counts().0, compiles);
}

#[test]
fn ordinary_buffer_edits_keep_closed_inputs_in_the_accepted_epoch() {
    let temp = test_support::TempDir::new("input-epoch-buffer-edit");
    let main = temp.path().join("main.jai");
    let dependency = temp.path().join("dependency.jai");
    let source =
        "#load \"dependency.jai\"; main :: () { value: Value; value = 1; if value == 1 {} }";
    std::fs::write(&main, source).unwrap();
    std::fs::write(&dependency, "Value :: int;").unwrap();
    let fs = Rc::new(CountingFs {
        path: test_support::lsp::native_identity(&dependency),
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
    assert!(session.diagnostics(&uri).unwrap().is_empty());
    fs.reads.set(0);
    std::fs::write(&dependency, "Value :: struct { field: int; };").unwrap();
    session
        .change(
            &uri,
            2,
            &[crate::TextChange {
                range: None,
                range_length: None,
                text: format!("{source} // edit"),
            }],
        )
        .unwrap();
    assert!(
        session.diagnostics(&uri).unwrap().is_empty(),
        "an edit uses the accepted closed-file epoch, not silent new disk bytes"
    );
    assert_eq!(session.document(&uri).unwrap().version, 2);
    assert_eq!(
        fs.reads.get(),
        0,
        "buffer events must not replay closed native inputs"
    );
    session.save(&uri).unwrap();
    assert!(
        !session.diagnostics(&uri).unwrap().is_empty(),
        "a source save is a known native event"
    );
    assert!(fs.reads.get() > 0, "saving starts a fresh native capture");
    std::fs::write(&dependency, "Value :: int;").unwrap();
    session.file_changed(&dependency, false);
    assert!(session.diagnostics(&uri).unwrap().is_empty());
}

#[test]
fn watched_saved_config_reads_after_native_event_invalidation() {
    let temp = test_support::TempDir::new("input-epoch-watched-config");
    let main = temp.path().join("main.jai");
    let config = temp.path().join("jacket.toml");
    let original = "[project]\nentry=[\"main.jai\"]\n";
    let changed = "[project]\nentry=[\"main.jai\"]\nimport_path=[\"next\"]\n";
    std::fs::write(&main, "main :: () {}").unwrap();
    std::fs::write(&config, original).unwrap();
    let mut session = Session::with_environment(
        crate::Limits::default(),
        crate::Environment {
            fs: Rc::new(NativeFs),
            services: None,
            options: Box::new(|_, _| {
                let mut options = jaic::sema::Options::host();
                options.runtime_support = false;
                options.stack_trace = false;
                options
            }),
        },
    );
    let config_uri = DocumentUri::from_path(&config).unwrap();
    let main_uri = DocumentUri::from_path(&main).unwrap();
    session
        .open(config_uri.clone(), 1, original.into())
        .unwrap();
    session
        .open(main_uri.clone(), 1, "main :: () {}".into())
        .unwrap();
    assert!(session.diagnostics(&main_uri).unwrap().is_empty());
    assert_eq!(session.saved_configs[&config_uri].text(), Some(original));
    std::fs::write(&config, changed).unwrap();
    session.saved_file_changed(&config, false);
    assert_eq!(
        session.saved_configs[&config_uri].text(),
        Some(changed),
        "a known disk event must not copy the old sealed configuration"
    );
    assert_eq!(
        session.lint_configs[&config_uri].text, original,
        "the watch event does not overwrite the editor buffer"
    );
}

#[cfg(unix)]
#[test]
fn watched_physical_config_deletion_removes_its_saved_alias_overlay() {
    let temp = test_support::TempDir::new("input-epoch-config-alias-delete");
    let main = temp.path().join("main.jai");
    let physical = temp.path().join("physical/jacket.toml");
    let alias = temp.path().join("jacket.toml");
    let text = "[project]\nentry=[\"main.jai\"]\n";
    std::fs::create_dir_all(physical.parent().unwrap()).unwrap();
    std::fs::write(&physical, text).unwrap();
    std::fs::write(&main, "main :: () {}").unwrap();
    std::os::unix::fs::symlink(&physical, &alias).unwrap();
    let mut session = Session::with_environment(
        crate::Limits::default(),
        crate::Environment {
            fs: Rc::new(NativeFs),
            services: None,
            options: Box::new(|_, _| {
                let mut options = jaic::sema::Options::host();
                options.runtime_support = false;
                options.stack_trace = false;
                options
            }),
        },
    );
    let config_uri = DocumentUri::from_path(&alias).unwrap();
    let main_uri = DocumentUri::from_path(&main).unwrap();
    session.open(config_uri.clone(), 1, text.into()).unwrap();
    session
        .open(main_uri.clone(), 1, "main :: () {}".into())
        .unwrap();
    assert!(session.diagnostics(&main_uri).unwrap().is_empty());
    assert!(session.saved_configs.contains_key(&config_uri));
    std::fs::remove_file(&physical).unwrap();
    session.saved_file_changed(&physical, true);
    assert!(
        matches!(
            session.saved_configs[&config_uri].state,
            crate::session::saved_configs::SavedConfigState::Missing
        ),
        "a dangling alias cannot retain its deleted physical config snapshot"
    );
    assert_eq!(
        session.lint_configs[&config_uri].text, text,
        "the editor buffer survives deletion independently of its saved overlay"
    );
    let recreated = "[project]\nentry=[\"other.jai\"]\n";
    std::fs::write(&physical, recreated).unwrap();
    session.saved_file_changed(&physical, true);
    assert_eq!(
        session.saved_configs[&config_uri].text(),
        Some(recreated),
        "a known recreation restores the saved snapshot through the current alias"
    );
    assert_eq!(session.lint_configs[&config_uri].text, text);
}

#[test]
fn watched_saved_config_growth_is_charged_without_replacing_the_editor_buffer() {
    let temp = test_support::TempDir::new("input-epoch-config-budget");
    let config = temp.path().join("jacket.toml");
    let original = "[project]\nentry=[]\n";
    std::fs::write(&config, original).unwrap();
    let mut session = Session::with_environment(
        crate::Limits {
            workspace_bytes: 4096,
            ..crate::Limits::default()
        },
        crate::Environment {
            fs: Rc::new(NativeFs),
            services: None,
            options: Box::new(|_, _| jaic::sema::Options::host()),
        },
    );
    let uri = DocumentUri::from_path(&config).unwrap();
    session.open(uri.clone(), 1, original.into()).unwrap();
    std::fs::write(&config, format!("{original}#{}", "x".repeat(8192))).unwrap();
    session.saved_file_changed(&config, false);
    assert!(
        session.saved_configs[&uri].unavailable() && session.saved_configs[&uri].text().is_none(),
        "an oversized native refresh cannot retain an uncharged saved snapshot"
    );
    assert_eq!(session.lint_configs[&uri].text, original);
    assert_eq!(session.lint_configs[&uri].version, 1);
}

#[test]
#[cfg(unix)]
fn matching_discovery_memo_rejects_unknown_projected_alias_without_rediscovery() {
    let temp = test_support::TempDir::new("epoch-memo-unknown-alias");
    let main = temp.path().join("main.jai");
    let alias = temp.path().join("new-alias.jai");
    let text = "main :: () {}";
    std::fs::write(&main, text).unwrap();
    let mut session = Session::with_environment(
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
    let uri = DocumentUri::from_path(&main).unwrap();
    session.open(uri.clone(), 1, text.into()).unwrap();
    assert!(session.diagnostics(&uri).unwrap().is_empty());
    let counts = session.compile_counts();
    assert!(session.input_epoch.as_ref().unwrap().ready());
    assert!(session.inputs.borrow().is_some());
    std::os::unix::fs::symlink(&main, &alias).unwrap();
    let alias_uri = DocumentUri::from_path(&alias).unwrap();
    assert!(
        session.compile_inputs(&alias_uri, text).is_none(),
        "a sealed unknown request identity is not a memo grant"
    );
    assert!(session.input_epoch.as_ref().unwrap().has_pending_coverage());
    assert!(
        session.inputs.borrow().is_some(),
        "pending hydration keeps its original discovery owner"
    );
    assert_eq!(
        session.compile_counts(),
        counts,
        "the failed selection cannot rediscover or compile inline"
    );
}


#[test]
fn first_workspace_admission_is_terminal_on_failure_and_warm_when_deferred() {
    for mode in ["pending", "cancel", "config"] {
        let temp = test_support::TempDir::new("epoch-workspace-admission-controls");
        let path = temp.path().join("main.jai");
        std::fs::write(&path, "main :: () {}").unwrap();
        let fs = Rc::new(ContextCountingFs(std::cell::Cell::new(0)));
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
        session.set_workspace_folders(vec![temp.path().to_path_buf()]);
        let uri = DocumentUri::from_path(&path).unwrap();
        session
            .open(uri.clone(), 1, "main :: () {}".into())
            .unwrap();
        assert!(session.diagnostics(&uri).unwrap().is_empty());
        session.defer_compiles(true);
        let epoch = session.input_epoch.as_ref().unwrap().clone();
        assert!(epoch.ready());
        let counts = session.compile_counts();
        let token = epoch.capture_identity();
        fs.0.set(0);
        if mode == "pending" {
            assert!(epoch.read(&temp.path().join("uncovered.input")).is_none());
            assert!(epoch.has_pending_coverage());
        }
        let rows = {
            let _scope = session.discovery_query();
            if mode == "cancel" {
                jaic::interrupt::arm();
                jaic::interrupt::request(true);
                assert!(jaic::interrupt::should_stop());
            }
            if mode == "config" {
                session.configuration_input_failed.set(true);
            }
            session.workspace_polymorphs(&uri)
        };
        if mode == "cancel" {
            let fired = jaic::interrupt::disarm();
            jaic::interrupt::request(false);
            assert!(fired);
        }
        assert!(rows.is_empty());
        assert_eq!(
            epoch.capture_identity(),
            token,
            "{mode} cannot reset an admitted failed context"
        );
        assert_eq!(fs.0.get(), 0, "{mode} failure performs no native fallback");
        assert_eq!(session.compile_counts(), counts);
        session.configuration_input_failed.set(false);
        for _ in 0..128 {
            if epoch.ready() {
                break;
            }
            session.reconcile_input_epoch();
        }
        assert!(epoch.ready());
        let before = epoch.capture_identity();
        {
            let _scope = session.discovery_query();
            assert!(session.workspace_polymorphs(&uri).is_empty());
        }
        assert_ne!(
            epoch.capture_identity(),
            before,
            "rejected admission never marked discovery accepted"
        );
        assert!(epoch.ready());
        assert_eq!(session.compile_counts(), counts);
        let accepted = epoch.capture_identity();
        fs.0.set(0);
        {
            let _scope = session.discovery_query();
            assert!(session.workspace_polymorphs(&uri).is_empty());
        }
        assert_eq!(epoch.capture_identity(), accepted);
        assert_eq!(
            fs.0.get(),
            0,
            "deferred empty summaries do not repeat native transactions"
        );
        assert_eq!(session.compile_counts(), counts);
        session.polymorph_index.borrow_mut().clear();
        {
            let _scope = session.discovery_query();
            assert!(session.workspace_polymorphs(&uri).is_empty());
        }
        assert_ne!(
            epoch.capture_identity(),
            accepted,
            "clear permits initial admission again"
        );
        assert!(epoch.ready());
        assert_eq!(session.compile_counts(), counts);
    }
}


#[test]
fn native_stdlib_workspace_summary_preserves_closed_callers() {
    let temp = test_support::lsp::project(
        "epoch-stdlib-workspace",
        &[(
            "main.jai",
            "#import \"Basic\"; main :: () {a:[..]u32; array_add(*a,1); b:[..]s64; array_add(*b,2);}",
        )],
    );
    let stdlib =
        std::fs::canonicalize(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../stdlib"))
            .unwrap();
    let path = stdlib.join("internal/basic/arrays.jai");
    let source = std::fs::read_to_string(&path).unwrap();
    let uri = DocumentUri::from_path(&path).unwrap();
    let fs = Rc::new(ContextCountingFs(std::cell::Cell::new(0)));
    let mut session = Session::with_environment(
        crate::Limits::default(),
        crate::Environment {
            fs: fs.clone(),
            services: None,
            options: Box::new(move |fs, _| {
                let mut options = jaic::sema::Options::host();
                options.import_paths = vec![stdlib.clone()];
                options.set_stdlib(fs, &stdlib);
                options
            }),
        },
    );
    session.set_workspace_folders(vec![temp.path().to_path_buf()]);
    session.open(uri.clone(), 1, source.clone()).unwrap();
    let lenses = session.code_lenses(&uri).unwrap();
    let declaration = source
        .find("array_add :: (array: *[..] $T, item: T)")
        .unwrap();
    let line = source[..declaration]
        .bytes()
        .filter(|b| *b == b'\n')
        .count() as u32;
    let epoch = session.input_epoch.as_ref().unwrap();
    let row = lenses.iter().find(|lens| lens.range.start.line == line).unwrap_or_else(|| panic!("array_add missing: ready={} recapture={} pending={} bytes={} native={} identities={:?} config_failed={} counts={:?}", epoch.ready(), epoch.needs_recapture(), epoch.has_pending_coverage(), epoch.retained_bytes(), fs.0.get(), session.documents.values().map(|d|d.identity_pending).collect::<Vec<_>>(),session.configuration_input_failed.get(),session.compile_counts()));
    let values = session.polymorphs(&uri, row.range.start);
    assert!(
        values.contains(&"T = u32".into()) && values.contains(&"T = s64".into()),
        "{values:?}"
    );
}
