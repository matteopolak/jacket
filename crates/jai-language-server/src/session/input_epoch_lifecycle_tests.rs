use super::*;

#[test]
fn cold_program_contexts_capture_once_and_warm_switches_do_no_native_work() {
    for loaded in [false, true] {
        let temp = test_support::TempDir::new("epoch-program-contexts");
        let main = temp.path().join("main.jai");
        let other = temp.path().join("other.jai");
        let source = if loaded {
            "#load \"other.jai\"; a :: () {}"
        } else {
            "a :: () {}"
        };
        std::fs::write(&main, source).unwrap();
        std::fs::write(&other, "b :: () {}").unwrap();
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
        let uri = DocumentUri::from_path(&main).unwrap();
        let other = DocumentUri::from_path(&other).unwrap();
        session.open(uri.clone(), 1, source.into()).unwrap();
        session.open(other.clone(), 1, "b :: () {}".into()).unwrap();
        assert!(session.diagnostics(&uri).unwrap().is_empty());
        assert!(session.diagnostics(&other).unwrap().is_empty());
        let counts = session.compile_counts();
        fs.0.set(0);
        for _ in 0..3 {
            for selected in [&uri, &other] {
                assert!(session.diagnostics(selected).unwrap().is_empty());
            }
        }
        assert_eq!(session.compile_counts(), counts);
        assert_eq!(
            fs.0.get(),
            0,
            "warmed program/loaded-child switches retain the sealed native view"
        );
    }
}

#[test]
fn cached_alternate_context_queries_retain_the_sealed_native_view() {
    let temp = test_support::TempDir::new("epoch-alternate-context");
    let main = temp.path().join("main.jai");
    let text = "#import \"Basic\"; #if OS == .WINDOWS { Windows_Only :: struct { value: int; } } main :: () { #if OS == .WINDOWS { marker: Windows_Only; } }";
    std::fs::write(&main, text).unwrap();
    let stdlib = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../stdlib");
    let fs = Rc::new(ContextCountingFs(std::cell::Cell::new(0)));
    let mut session = Session::with_environment(
        crate::Limits::default(),
        crate::Environment {
            fs: fs.clone(),
            services: None,
            options: Box::new(move |fs, _| {
                let mut options = jaic::sema::Options::host();
                options.os = jaic::target::TargetOs::Linux;
                options.runtime_support = false;
                options.stack_trace = false;
                options.import_paths = vec![stdlib.clone()];
                options.set_stdlib(fs, &stdlib);
                options
            }),
        },
    );
    let uri = DocumentUri::from_path(&main).unwrap();
    session.open(uri.clone(), 1, text.into()).unwrap();
    session.diagnostics(&uri).unwrap();
    let byte = text.rfind("Windows_Only").unwrap();
    assert!(
        session
            .with_semantic_at(&uri, text, byte, semantic::Analysis::hover)
            .is_some()
    );
    let counts = session.compile_counts();
    fs.0.set(0);
    for _ in 0..3 {
        assert!(
            session
                .with_semantic_at(&uri, text, byte, semantic::Analysis::hover)
                .is_some()
        );
        assert!(session.input_epoch.as_ref().unwrap().ready());
    }
    assert_eq!(session.compile_counts(), counts);
    assert_eq!(
        fs.0.get(),
        0,
        "a cached alternate target is not a cold native capture"
    );
}

#[test]
fn advisory_catalogue_warming_preserves_diagnostics_actions_and_tokens() {
    let temp = test_support::TempDir::new("epoch-advisory-native-client");
    let main = temp.path().join("lint.jai");
    let text = include_str!("../../../../editors/vscode/test/e2e/fixture/lint.jai");
    std::fs::write(&main, text).unwrap();
    std::fs::write(temp.path().join("jacket.toml"), "[lint.rules]\n").unwrap();
    let stdlib = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../stdlib");
    let mut session = Session::with_environment(
        crate::Limits::default(),
        crate::Environment {
            fs: Rc::new(NativeFs),
            services: None,
            options: Box::new(move |fs, _| {
                let mut options = jaic::sema::Options::host();
                options.runtime_support = false;
                options.stack_trace = false;
                options.import_paths = vec![stdlib.clone()];
                options.set_stdlib(fs, &stdlib);
                options
            }),
        },
    );
    let uri = DocumentUri::from_path(&main).unwrap();
    session.open(uri.clone(), 1, text.into()).unwrap();
    let diagnostics = session.diagnostics(&uri).unwrap();
    assert!(
        diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == crate::DiagnosticCode::Lint("bool_comparison"))
    );
    let counts = session.compile_counts();
    let retained = session.input_epoch.as_ref().unwrap().retained_bytes();
    session.warm_auto_import();
    assert_eq!(
        session.input_epoch.as_ref().unwrap().retained_bytes(),
        retained,
        "advisory catalogue bytes do not enter semantic or strict hint input authority"
    );
    assert!(
        session.input_epoch.as_ref().unwrap().ready(),
        "optional catalogue warming must not queue unknown inputs in the accepted semantic epoch"
    );
    let actions = session
        .code_actions_in(
            &uri,
            crate::Range::new(crate::Position::new(0, 0), crate::Position::new(12, 0)),
            &crate::lints::ActionContext {
                only: Some(vec![crate::lints::FIX_ALL_KIND.into()]),
                diagnostics: Vec::new(),
            },
        )
        .unwrap();
    assert!(
        actions
            .iter()
            .any(|action| action.kind == Some(crate::lints::FIX_ALL_KIND) && action.edit.is_some())
    );
    assert!(!session.semantic_tokens(&uri).unwrap().is_empty());
    assert_eq!(session.diagnostics(&uri).unwrap(), diagnostics);
    assert_eq!(session.compile_counts(), counts);
    let plain = temp.path().join("auto-import.jai");
    let text = "main :: () {\n    prin\n}\n";
    std::fs::write(&plain, text).unwrap();
    let plain = DocumentUri::from_path(&plain).unwrap();
    session.open(plain.clone(), 1, text.into()).unwrap();
    let completion = session
        .completion(&plain, crate::Position::new(1, 8))
        .unwrap();
    assert!(
        completion.items.iter().any(|item| item.label == "print"
            && item
                .additional_edits
                .iter()
                .any(|edit| edit.new_text.contains("#import \"Basic\";"))),
        "accepted catalogue warming must preserve real unloaded-module completion"
    );
}

#[test]
fn sequential_buffer_open_keeps_previous_program_covered_for_deferred_clients() {
    let temp = test_support::TempDir::new("epoch-sequential-editor-open");
    let first = temp.path().join("broken.jai");
    let second = temp.path().join("lint.jai");
    let broken = "main :: () { value: int = \"text\"; }";
    let lint = include_str!("../../../../editors/vscode/test/e2e/fixture/lint.jai");
    std::fs::write(&first, broken).unwrap();
    std::fs::write(&second, lint).unwrap();
    let stdlib = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../stdlib");
    let mut session = Session::with_environment(
        crate::Limits::default(),
        crate::Environment {
            fs: Rc::new(NativeFs),
            services: None,
            options: Box::new(move |fs, _| {
                let mut options = jaic::sema::Options::host();
                options.runtime_support = false;
                options.stack_trace = false;
                options.import_paths = vec![stdlib.clone()];
                options.set_stdlib(fs, &stdlib);
                options
            }),
        },
    );
    session.defer_compiles(true);
    let first = DocumentUri::from_path(&first).unwrap();
    let second = DocumentUri::from_path(&second).unwrap();
    session.open(first.clone(), 1, broken.into()).unwrap();
    let expected = session.diagnostics(&first).unwrap();
    assert!(!expected.is_empty());
    session.open(second.clone(), 1, lint.into()).unwrap();
    assert!(
        session
            .diagnostics(&second)
            .unwrap()
            .iter()
            .any(|diagnostic| diagnostic.code == crate::DiagnosticCode::Lint("bool_comparison"))
    );
    assert_eq!(
        session.diagnostics(&first),
        Ok(expected),
        "opening another buffer must not remove accepted closed inputs from the first program"
    );
    let actions = session
        .code_actions_in(
            &second,
            crate::Range::new(crate::Position::new(0, 0), crate::Position::new(12, 0)),
            &crate::lints::ActionContext {
                only: Some(vec![crate::lints::FIX_ALL_KIND.into()]),
                diagnostics: Vec::new(),
            },
        )
        .unwrap();
    assert!(
        actions
            .iter()
            .any(|action| action.kind == Some(crate::lints::FIX_ALL_KIND) && action.edit.is_some())
    );
    assert!(!session.semantic_tokens(&second).unwrap().is_empty());
}

#[test]
fn pending_input_coverage_is_not_reported_as_an_accepted_background_compile() {
    let temp = test_support::TempDir::new("epoch-pending-compile-status");
    let path = temp.path().join("main.jai");
    let text = "main :: () {}";
    std::fs::write(&path, text).unwrap();
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
    let uri = DocumentUri::from_path(&path).unwrap();
    session.open(uri.clone(), 1, text.into()).unwrap();
    assert!(session.diagnostics(&uri).unwrap().is_empty());
    let counts = session.compile_counts();
    assert!(
        !session
            .input_epoch
            .as_ref()
            .unwrap()
            .is_file(&temp.path().join("unknown"))
    );
    assert!(session.input_epoch.as_ref().unwrap().has_pending_coverage());
    session.want_compile(&uri, text);
    assert_eq!(
        session.warm_pending(),
        None,
        "an unavailable callback cannot schedule successful-compile refreshes"
    );
    assert!(
        session.compile_pending(),
        "demonstrated coverage work can resume in a later transaction"
    );
    assert_eq!(session.compile_counts(), counts);
    for _ in 0..128 {
        session.reconcile_input_epoch();
        if session.input_epoch.as_ref().unwrap().ready() {
            break;
        }
    }
    assert!(session.input_epoch.as_ref().unwrap().ready());
    assert_eq!(session.warm_pending(), Some(true));
    assert_eq!(session.compile_counts(), counts);
}

#[cfg(unix)]
#[test]
fn pending_epoch_alias_open_does_not_prove_a_distinct_buffer() {
    let temp = test_support::TempDir::new("epoch-pending-open-alias");
    let path = temp.path().join("main.jai");
    let alias = temp.path().join("alias.jai");
    let text = "main :: () {}";
    std::fs::write(&path, text).unwrap();
    std::os::unix::fs::symlink(&path, &alias).unwrap();
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
    let uri = DocumentUri::from_path(&path).unwrap();
    let alias = DocumentUri::from_path(&alias).unwrap();
    session.open(uri.clone(), 1, text.into()).unwrap();
    assert!(session.diagnostics(&uri).unwrap().is_empty());
    let epoch = session.input_epoch.as_ref().unwrap().clone();
    assert!(!epoch.is_file(&temp.path().join("unknown")));
    assert!(epoch.has_pending_coverage());
    session.open(alias.clone(), 2, text.into()).unwrap();
    assert_eq!(session.documents.len(), 2);
    assert!(session.documents[&alias].identity_pending);
    assert_eq!(
        session.diagnostics(&alias),
        Err(crate::Error::DiagnosticsUnavailable)
    );
    assert_eq!(session.documents.get(&uri).unwrap().version, 1);
    assert!(epoch.has_pending_coverage());
    for _ in 0..128 {
        session.reconcile_input_epoch();
    }
    assert!(session.documents[&alias].identity_pending);
    session.close(&alias).unwrap();
    assert_eq!(session.documents.len(), 1);
    session.close(&uri).unwrap();
    assert!(session.documents.is_empty());
    assert!(
        epoch.ready(),
        "raw closes retain the accepted closed-input map"
    );
}

#[test]
fn pending_and_failed_parent_keep_json_open_changes_until_identity_recovery() {
    for failed in [false, true] {
        let temp = test_support::TempDir::new("epoch-json-pending-open");
        let first = temp.path().join("first.jai");
        let second = temp.path().join("second.jai");
        let marker = temp.path().join("marker");
        std::fs::write(&first, "first :: () {}").unwrap();
        std::fs::write(&second, "second :: () {}").unwrap();
        std::fs::write(&marker, "marker").unwrap();
        let mut json = crate::JsonSession::with_environment(
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
        json.defer_publications(true);
        let mut send = |value: serde_json::Value| {
            json.handle_json(&value.to_string()).unwrap();
        };
        send(
            serde_json::json!({"jsonrpc":"2.0","id":1,"method":"initialize","params":{"capabilities":{}}}),
        );
        send(serde_json::json!({"jsonrpc":"2.0","method":"initialized","params":{}}));
        let first = DocumentUri::from_path(&first).unwrap();
        let second = DocumentUri::from_path(&second).unwrap();
        send(
            serde_json::json!({"jsonrpc":"2.0","method":"textDocument/didOpen","params":{"textDocument":{"uri":first.as_str(),"languageId":"jai","version":1,"text":"first :: () {}"}}}),
        );
        assert!(json.session().diagnostics(&first).unwrap().is_empty());
        let epoch = json.session().input_epoch.as_ref().unwrap().clone();
        if failed {
            epoch.begin_capture();
            assert!(epoch.is_file(&marker));
            std::fs::remove_file(&marker).unwrap();
            assert!(!epoch.is_file(&marker));
            assert!(!epoch.validate_actual());
        } else {
            assert!(!epoch.is_file(&temp.path().join("unknown")));
        }
        json.handle_json(&serde_json::json!({"jsonrpc":"2.0","method":"textDocument/didOpen","params":{"textDocument":{"uri":second.as_str(),"languageId":"jai","version":1,"text":"second :: () {}"}}}).to_string()).unwrap();
        json.handle_json(&serde_json::json!({"jsonrpc":"2.0","method":"textDocument/didChange","params":{"textDocument":{"uri":second.as_str(),"version":2},"contentChanges":[{"text":"second :: () { } // version two"}]}}).to_string()).unwrap();
        assert_eq!(
            json.session().document_text(&second).unwrap(),
            "second :: () { } // version two"
        );
        assert_eq!(json.session().documents[&second].version, 2);
        assert!(json.session().documents[&second].identity_pending);
        assert_eq!(
            json.session().diagnostics(&second),
            Err(crate::Error::DiagnosticsUnavailable)
        );
        assert!(
            json.has_input_epoch_work(),
            "unresolved raw notifications must receive native-loop service opportunities"
        );
        for _ in 0..256 {
            json.reconcile_input_epoch_messages().unwrap();
            if !json.session().documents[&second].identity_pending {
                break;
            }
        }
        assert!(
            !json.session().documents[&second].identity_pending,
            "raw notification identity must make service progress (failed={failed})"
        );
        assert!(json.session().diagnostics(&second).unwrap().is_empty());
    }
}

#[test]
fn saved_settings_roundtrip_keeps_later_editor_completion_and_tokens_useful() {
    let cases = vec![
        (false, false, false),
        (true, false, false),
        (true, true, false),
    ];
    #[cfg(unix)]
    let cases = {
        let mut cases = cases;
        cases.push((true, true, true));
        cases
    };
    for (native_options, keep_config, aliased) in cases {
        let temp = test_support::TempDir::new("epoch-editor-settings-roundtrip");
        let alias_owner = test_support::TempDir::new("epoch-editor-client-spelling");
        let client_root = if aliased {
            let alias = alias_owner.path().join("workspace");
            #[cfg(unix)]
            std::os::unix::fs::symlink(temp.path(), &alias).unwrap();
            alias
        } else {
            temp.path().to_owned()
        };
        let watcher = client_root.join("jacket.toml");
        let lint_text = include_str!("../../../../editors/vscode/test/e2e/fixture/lint.jai");
        let plain_text =
            include_str!("../../../../editors/vscode/test/e2e/fixture/auto-import.jai");
        let settings_text = include_str!("../../../../editors/vscode/test/e2e/fixture/jacket.toml");
        let lint = temp.path().join("lint.jai");
        let plain = temp.path().join("auto-import.jai");
        let settings = temp.path().join("jacket.toml");
        std::fs::write(&lint, lint_text).unwrap();
        std::fs::write(&plain, plain_text).unwrap();
        std::fs::write(&settings, settings_text).unwrap();
        let stdlib = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../stdlib");
        let mut session = Session::with_environment(
            crate::Limits::default(),
            crate::Environment {
                fs: Rc::new(NativeFs),
                services: native_options.then(|| {
                    Rc::new(crate::services::EditorServices::new(
                        jaic::service::Services::new(jaic::service::ServiceConfig::default()),
                    )) as Rc<dyn jaic::service::ServiceProvider>
                }),
                options: Box::new(move |fs, main| {
                    if native_options {
                        let inputs = jaic::project::ProgramInputs::for_file(fs, main, &stdlib);
                        return jaic::sema::Options::for_program(fs, main, &inputs);
                    }
                    let mut options = jaic::sema::Options::host();
                    options.runtime_support = false;
                    options.stack_trace = false;
                    options.import_paths = vec![stdlib.clone()];
                    options.set_stdlib(fs, &stdlib);
                    options
                }),
            },
        );
        session.defer_compiles(true);
        session.set_workspace_folders(vec![client_root.clone()]);
        let lint = DocumentUri::from_path(&client_root.join("lint.jai")).unwrap();
        let plain = DocumentUri::from_path(&client_root.join("auto-import.jai")).unwrap();
        let config = DocumentUri::from_path(&client_root.join("jacket.toml")).unwrap();
        session.open(lint.clone(), 1, lint_text.into()).unwrap();
        assert!(
            session
                .diagnostics(&lint)
                .unwrap()
                .iter()
                .any(|d| d.code == crate::DiagnosticCode::Lint("bool_comparison"))
        );
        session
            .open(config.clone(), 1, settings_text.into())
            .unwrap();
        let allow = settings_text.replacen(
            "[lint.rules]",
            "[lint.rules]\nbool_comparison = \"allow\"",
            1,
        );
        session
            .change(
                &config,
                2,
                &[crate::TextChange {
                    range: None,
                    range_length: None,
                    text: allow.clone(),
                }],
            )
            .unwrap();
        std::fs::write(&settings, &allow).unwrap();
        session.save(&config).unwrap();
        assert!(
            !session
                .diagnostics(&lint)
                .unwrap()
                .iter()
                .any(|d| d.code == crate::DiagnosticCode::Lint("bool_comparison"))
        );
        if keep_config {
            session.file_changed(&watcher, false);
            session.file_changed(&watcher, false);
        }
        session
            .change(
                &config,
                3,
                &[crate::TextChange {
                    range: None,
                    range_length: None,
                    text: settings_text.into(),
                }],
            )
            .unwrap();
        std::fs::write(&settings, settings_text).unwrap();
        session.save(&config).unwrap();
        if !keep_config {
            session.close(&config).unwrap();
        }
        session.open(plain.clone(), 1, plain_text.into()).unwrap();
        // The real client pulls diagnostics for the newly opened incomplete file before
        // requesting completion. Optional typo help must not revoke other semantic inputs.
        let _typo_diagnostics = session.diagnostics(&plain);
        for _ in 0..128 {
            session.reconcile_input_epoch();
        }
        assert!(
            session.input_epoch.as_ref().unwrap().ready(),
            "the incomplete-file diagnostic cannot poison the shared semantic epoch"
        );
        let completion = session
            .completion(&plain, crate::Position::new(1, 8))
            .unwrap();
        assert!(
            completion.items.iter().any(|item| item.label == "print"
                && item
                    .additional_edits
                    .iter()
                    .any(|edit| edit.new_text == "#import \"Basic\";\n\n")),
            "settings lifecycle must preserve unloaded-module import completion"
        );
        if keep_config {
            // The SDK watcher reports the restored save after the completion request,
            // and both registered watchers can report the same physical change.
            session.file_changed(&watcher, false);
            session.file_changed(&watcher, false);
        }
        if session.compile_pending() {
            let _ = session.warm_pending();
        }
        for _ in 0..128 {
            session.reconcile_input_epoch();
        }
        assert!(!session.semantic_tokens(&lint).unwrap().is_empty());
    }
}

#[test]
fn pending_open_identity_hands_oversized_required_inputs_to_strict_native_capture() {
    let temp = test_support::TempDir::new("epoch-pending-open-large-input");
    let main = temp.path().join("main.jai");
    let second = temp.path().join("second.jai");
    let data = temp.path().join("options.data");
    std::fs::write(&main, "main :: () {}").unwrap();
    std::fs::write(&second, "second :: () {}").unwrap();
    std::fs::write(&data, vec![b'x'; 17 * 1024 * 1024]).unwrap();
    let enabled = Rc::new(std::cell::Cell::new(false));
    let read_data = enabled.clone();
    let options_data = data.clone();
    let fs = Rc::new(CountingFs {
        path: test_support::lsp::native_identity(&data),
        reads: std::cell::Cell::new(0),
    });
    let mut session = Session::with_environment(
        crate::Limits::default(),
        crate::Environment {
            fs: fs.clone(),
            services: None,
            options: Box::new(move |fs, _| {
                let mut options = jaic::sema::Options::host();
                options.runtime_support = false;
                options.stack_trace = read_data.get()
                    && fs
                        .read(&options_data)
                        .is_some_and(|bytes| bytes.first() == Some(&b'y'));
                options
            }),
        },
    );
    let uri = DocumentUri::from_path(&main).unwrap();
    let second = DocumentUri::from_path(&second).unwrap();
    session
        .open(uri.clone(), 1, "main :: () {}".into())
        .unwrap();
    assert!(session.diagnostics(&uri).unwrap().is_empty());
    let counts = session.compile_counts();
    let epoch = session.input_epoch.as_ref().unwrap().clone();
    assert!(epoch.read_shared(&data).is_none());
    for _ in 0..128 {
        session.reconcile_input_epoch();
    }
    assert_eq!(
        fs.reads.get(),
        1,
        "oversized unowned hydration is attempted once"
    );
    assert!(!epoch.ready());
    assert!(!epoch.has_pending_coverage());
    session
        .open(second.clone(), 1, "second :: () {}".into())
        .unwrap();
    assert!(session.documents[&second].identity_pending);
    for _ in 0..256 {
        session.reconcile_input_epoch();
        if !session.documents[&second].identity_pending {
            break;
        }
    }
    assert!(!session.documents[&second].identity_pending);
    assert!(
        !epoch.ready(),
        "identity-only completion cannot seal over an oversized required input"
    );
    assert_eq!(
        fs.reads.get(),
        1,
        "identity service does not retry oversized payload hydration"
    );
    enabled.set(true);
    assert!(
        session.diagnostics(&second).unwrap().is_empty(),
        "later strict capture must accept the complete options input with its proof owner"
    );
    assert_eq!(session.compile_counts().0, counts.0 + 1);
    assert_eq!(session.compile_counts().1, counts.1 + 1);
    assert!(
        epoch.ready(),
        "the actual large-read proof owner can now seal the complete epoch"
    );
    let accepted_counts = session.compile_counts();
    let accepted_reads = fs.reads.get();
    assert!(session.diagnostics(&second).unwrap().is_empty());
    assert_eq!(session.compile_counts(), accepted_counts);
    assert_eq!(
        fs.reads.get(),
        accepted_reads,
        "warm accepted ownership does not rehydrate the payload"
    );
}

#[test]
fn known_open_buffer_keeps_syntax_tokens_while_unrelated_input_coverage_is_pending() {
    let temp = test_support::TempDir::new("epoch-current-buffer-syntax-tokens");
    let path = temp.path().join("main.jai");
    let text = "main :: () { value := 42; if value == 42 {} }";
    std::fs::write(&path, text).unwrap();
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
    let uri = DocumentUri::from_path(&path).unwrap();
    session.open(uri.clone(), 1, text.into()).unwrap();
    assert!(session.diagnostics(&uri).is_ok());
    assert!(!session.semantic_tokens(&uri).unwrap().is_empty());
    let counts = session.compile_counts();
    let epoch = session.input_epoch.as_ref().unwrap();
    assert!(!epoch.is_file(&temp.path().join("unrelated-unknown")));
    assert!(epoch.has_pending_coverage());
    let tokens = session.semantic_tokens(&uri).unwrap();
    assert!(
        !tokens.is_empty(),
        "current-buffer syntax tokens do not require unrelated native coverage"
    );
    assert!(
        tokens
            .iter()
            .any(|token| token.kind == crate::SemanticTokenKind::Number)
    );
    assert_eq!(session.compile_counts(), counts);
    assert_eq!(session.documents[&uri].version, 1);
    assert!(epoch.has_pending_coverage());
}

#[test]
fn active_parent_hydration_cadence_stops_after_completion_or_unavailable_input() {
    let temp = test_support::TempDir::new("epoch-active-hydration-cadence");
    let main = temp.path().join("main.jai");
    let large = temp.path().join("large.data");
    std::fs::write(&main, "main :: () {}").unwrap();
    std::fs::write(&large, vec![b'x'; 17 * 1024 * 1024]).unwrap();
    let mut json = crate::JsonSession::with_environment(
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
    json.defer_publications(true);
    for value in [
        serde_json::json!({"jsonrpc":"2.0","id":1,"method":"initialize","params":{"capabilities":{}}}),
        serde_json::json!({"jsonrpc":"2.0","method":"initialized","params":{}}),
    ] {
        json.handle_json(&value.to_string()).unwrap();
    }
    let uri = DocumentUri::from_path(&main).unwrap();
    json.handle_json(&serde_json::json!({"jsonrpc":"2.0","method":"textDocument/didOpen","params":{"textDocument":{"uri":uri.as_str(),"languageId":"jai","version":1,"text":"main :: () {}"}}}).to_string()).unwrap();
    assert!(json.session().diagnostics(&uri).unwrap().is_empty());
    let epoch = json.session().input_epoch.as_ref().unwrap().clone();
    assert!(!json.input_epoch_hydration_pending());
    assert!(!epoch.is_file(&temp.path().join("unknown")));
    assert!(
        json.input_epoch_hydration_pending(),
        "queued coverage selects immediate bounded owner service"
    );
    for _ in 0..128 {
        json.reconcile_input_epoch_messages().unwrap();
        if epoch.ready() {
            break;
        }
    }
    assert!(epoch.ready());
    assert!(
        !json.input_epoch_hydration_pending(),
        "completion returns to ordinary polling"
    );
    assert!(epoch.read_shared(&large).is_none());
    assert!(json.input_epoch_hydration_pending());
    for _ in 0..128 {
        json.reconcile_input_epoch_messages().unwrap();
        if !epoch.has_pending_coverage() {
            break;
        }
    }
    assert!(!epoch.ready());
    assert!(
        !json.input_epoch_hydration_pending(),
        "terminal oversized input cannot create an immediate native retry loop"
    );
    // A new known native event starts the independent shutdown-gating scenario.
    epoch.begin_capture();
    assert!(epoch.seal_after_validation());
    assert!(!epoch.is_file(&temp.path().join("another")));
    assert!(json.input_epoch_hydration_pending());
    json.handle_json(&serde_json::json!({"jsonrpc":"2.0","id":2,"method":"shutdown"}).to_string())
        .unwrap();
    assert!(
        !json.input_epoch_hydration_pending(),
        "shutdown never requests eager reconciliation"
    );
}

#[test]
fn known_open_buffer_keeps_pure_extract_edit_while_unrelated_coverage_is_pending() {
    let temp = test_support::TempDir::new("epoch-current-buffer-extract");
    let path = temp.path().join("refactor.jai");
    let text = include_str!("../../../../editors/vscode/test/e2e/fixture/refactor.jai");
    std::fs::write(&path, text).unwrap();
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
    let uri = DocumentUri::from_path(&path).unwrap();
    session.open(uri.clone(), 1, text.into()).unwrap();
    assert!(session.diagnostics(&uri).unwrap().is_empty());
    let line = text.lines().nth(5).unwrap();
    let from = line.find("leaf(n)").unwrap() as u32;
    let range = crate::Range::new(
        crate::Position::new(5, from),
        crate::Position::new(5, from + 7),
    );
    let context = crate::lints::ActionContext {
        only: Some(vec!["refactor.extract".into()]),
        diagnostics: Vec::new(),
    };
    let expected = session
        .code_actions_in(&uri, range, &context)
        .unwrap()
        .into_iter()
        .find(|action| action.title == "Extract into variable")
        .expect("useful initial extract");
    let counts = session.compile_counts();
    let epoch = session.input_epoch.as_ref().unwrap();
    assert!(!epoch.is_file(&temp.path().join("unrelated-unknown")));
    assert!(epoch.has_pending_coverage());
    let action = session
        .code_actions_in(&uri, range, &context)
        .unwrap()
        .into_iter()
        .find(|action| action.title == "Extract into variable");
    assert_eq!(
        action,
        Some(expected),
        "current-buffer pure extraction is independent of unrelated native coverage"
    );
    assert_eq!(session.compile_counts(), counts);
    assert!(epoch.has_pending_coverage());
}

#[test]
fn prepare_rename_recovery_retires_only_the_rejected_initial_cache_candidate() {
    struct RewriteStage {
        path: PathBuf,
        reads: std::cell::Cell<usize>,
        rewrite_at: std::cell::Cell<Option<usize>>,
        owner: std::cell::RefCell<std::rc::Weak<Session>>,
        cache_borrowed_at_rewrite: std::cell::Cell<Option<bool>>,
    }
    impl FileSystem for RewriteStage {
        fn read(&self, path: &Path) -> Option<Vec<u8>> {
            if path == self.path {
                let n = self.reads.get() + 1;
                self.reads.set(n);
                if self.rewrite_at.get() == Some(n) {
                    self.rewrite_at.set(None);
                    let owner = self.owner.borrow().upgrade().unwrap();
                    self.cache_borrowed_at_rewrite
                        .set(Some(owner.semantic.try_borrow_mut().is_err()));
                    std::fs::write(&self.path, "evil :: () {}\nleaf :: () {}\n").unwrap();
                }
            }
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
        fn canonical(&self, path: &Path) -> PathBuf {
            NativeFs.canonical(path)
        }
        fn canonical_exact(&self, path: &Path) -> Option<PathBuf> {
            NativeFs.canonical_exact(path)
        }
        fn is_native(&self) -> bool {
            true
        }
    }
    let temp = test_support::TempDir::new("epoch-rename-recovery-stage");
    let events = test_support::TempDir::new("epoch-rename-admission-stage");
    let marker = events.path().join("event");
    std::fs::write(&marker, "known native event").unwrap();
    let main = temp.path().join("main.jai");
    let callee = temp.path().join("callee.jai");
    let text = "#load \"callee.jai\"; main :: () { leaf(); }\n";
    std::fs::write(&main, text).unwrap();
    std::fs::write(&callee, "leaf :: () {}\n").unwrap();
    let fs = Rc::new(RewriteStage {
        path: test_support::lsp::native_identity(&callee),
        reads: std::cell::Cell::new(0),
        rewrite_at: std::cell::Cell::new(None),
        owner: std::cell::RefCell::default(),
        cache_borrowed_at_rewrite: std::cell::Cell::new(None),
    });
    let mut session = Session::with_environment(
        crate::Limits::default(),
        crate::Environment {
            fs: fs.clone(),
            services: None,
            options: Box::new(|_, _| {
                let mut o = jaic::sema::Options::host();
                o.runtime_support = false;
                o.stack_trace = false;
                o
            }),
        },
    );
    let uri = DocumentUri::from_path(&main).unwrap();
    session.open(uri.clone(), 1, text.into()).unwrap();
    assert!(session.diagnostics(&uri).unwrap().is_empty());
    let position = crate::Position::new(0, text.find("leaf").unwrap() as u32 + 1);
    let expected = session.prepare_rename(&uri, position).unwrap();
    assert!(expected.is_some());
    session.file_changed(&marker, false);
    fs.reads.set(0);
    assert_eq!(session.prepare_rename(&uri, position).unwrap(), expected);
    assert!(fs.reads.get() >= 3);
    let (root, dirs) = {
        let inputs = session.inputs.borrow();
        let inputs = &inputs.as_ref().unwrap().3;
        (inputs.1.clone(), inputs.2.clone())
    };
    assert!(
        session
            .semantic
            .borrow()
            .has_accepted_context(&root, &dirs, None)
    );
    let counts = session.compile_counts();
    session.file_changed(&marker, false);
    fs.reads.set(0);
    fs.rewrite_at.set(Some(3));
    let session = Rc::new(session);
    *fs.owner.borrow_mut() = Rc::downgrade(&session);
    assert_eq!(session.prepare_rename(&uri, position).unwrap(), None);
    assert_eq!(fs.rewrite_at.get(), None);
    assert_eq!(session.compile_counts(), counts);
    assert_eq!(
        fs.cache_borrowed_at_rewrite.get(),
        Some(false),
        "the controlled rewrite precedes initial cache selection"
    );
    assert!(
        session
            .semantic
            .borrow()
            .has_accepted_context(&root, &dirs, None),
        "the failing admission has not selected or globally retired the old compiler"
    );
    assert!(session.input_epoch.as_ref().unwrap().needs_recapture());
    assert!(!session.input_epoch.as_ref().unwrap().has_pending_coverage());
    assert!(session.inputs.borrow().is_none());
    assert_eq!(session.prepare_rename(&uri, position).unwrap(), expected);
    assert_eq!(session.compile_counts().0, counts.0 + 1);
    assert!(
        session.input_epoch.as_ref().unwrap().ready(),
        "the useful replacement accepted a consistent fresh capture"
    );
}
