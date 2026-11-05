//! Watched disk dependencies must retire checked answers even when overlays do not change.
use jai_language_server::{DiagnosticSeverity, DocumentUri, Session};
use test_support::lsp::{lean_session, uri_of};

struct Project {
    _temp: test_support::TempDir,
    root: std::path::PathBuf,
}

impl Project {
    fn path(&self) -> &std::path::Path {
        &self.root
    }
}

// Missing-name help searches nearby directories. Keep every consulted ancestor private.
fn project(name: &str, files: &[(&str, &str)]) -> Project {
    let temp = test_support::TempDir::new(name);
    let root = temp.path().join("workspace/isolated/project/source");
    std::fs::create_dir_all(&root).unwrap();
    for (path, text) in files {
        let path = root.join(path);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, text).unwrap();
    }
    Project {
        _temp: temp,
        root,
    }
}

fn errors(session: &Session, uri: &DocumentUri) -> usize {
    session
        .diagnostics(uri)
        .unwrap()
        .iter()
        .filter(|d| d.severity == DiagnosticSeverity::Error)
        .count()
}

#[test]
fn watched_loaded_file_edit_delete_and_recreate_are_rechecked() {
    let dir = project(
        "disk-freshness",
        &[
            (
                "main.jai",
                "#load \"dependency.jai\";\nmain :: () { value: int = supplied; }\n",
            ),
            ("dependency.jai", "supplied :: 42;\n"),
        ],
    );
    let root = dir.path().join("main.jai");
    let dependency = dir.path().join("dependency.jai");
    let uri = uri_of(&root);
    let mut session = lean_session();
    session
        .open(uri.clone(), 1, std::fs::read_to_string(&root).unwrap())
        .unwrap();
    assert_eq!(errors(&session, &uri), 0);
    let compiled = session.compile_counts().0;
    std::fs::write(&dependency, "supplied :: \"wrong type\";\n").unwrap();
    session.file_changed(&dependency, false);
    assert!(
        errors(&session, &uri) > 0,
        "edited loaded declaration must affect root diagnostics"
    );
    assert!(session.compile_counts().0 > compiled);
    std::fs::remove_file(&dependency).unwrap();
    session.file_changed(&dependency, true);
    assert!(errors(&session, &uri) > 0, "deleted load must be diagnosed");
    std::fs::write(&dependency, "supplied :: 42;\n").unwrap();
    session.file_changed(&dependency, true);
    assert!(
        errors(&session, &uri) == 0,
        "recreated dependency must repair diagnostics"
    );
}

#[test]
fn watched_unrelated_file_does_not_discard_a_checked_program() {
    let dir = project("disk-unrelated", &[("main.jai", "main :: () {}\n")]);
    let root = dir.path().join("main.jai");
    let uri = uri_of(&root);
    let mut session = lean_session();
    session
        .open(uri.clone(), 1, std::fs::read_to_string(root).unwrap())
        .unwrap();
    session.diagnostics(&uri).unwrap();
    let counts = session.compile_counts();
    session.file_changed(&dir.path().join("unread.txt"), false);
    session.diagnostics(&uri).unwrap();
    assert_eq!(session.compile_counts(), counts);
}

#[test]
fn a_failed_load_is_rechecked_when_the_file_appears() {
    let dir = project(
        "disk-created",
        &[(
            "main.jai",
            "#load \"later.jai\"; main :: () { supplied(); }\n",
        )],
    );
    let root = dir.path().join("main.jai");
    let dependency = dir.path().join("later.jai");
    let uri = uri_of(&root);
    let mut session = lean_session();
    session
        .open(uri.clone(), 1, std::fs::read_to_string(root).unwrap())
        .unwrap();
    assert!(errors(&session, &uri) > 0);
    std::fs::write(&dependency, "supplied :: () {}\n").unwrap();
    session.file_changed(&dependency, true);
    assert_eq!(errors(&session, &uri), 0);
}

#[test]
fn watched_compile_time_data_reads_are_rechecked() {
    use test_support::lsp::session;
    let dir = project("disk-run-read", &[("data.txt", "one")]);
    let data = dir.path().join("data.txt");
    let source = format!(
        "#import \"Basic\"; #import \"File\"; DATA :: #run read_entire_file(\"{}\"); main :: () {{ #assert DATA == \"one\"; }}",
        jacket_config::slashed(&data)
    );
    let uri = uri_of(&dir.path().join("main.jai"));
    let mut session = session();
    session.open(uri.clone(), 1, source).unwrap();
    assert_eq!(errors(&session, &uri), 0);
    std::fs::write(&data, "two").unwrap();
    session.file_changed(&data, false);
    assert!(errors(&session, &uri) > 0);
}

#[cfg(unix)]
#[test]
fn watched_leaf_symlink_delete_and_recreate_retire_target_facts() {
    let dir = project(
        "disk-symlink",
        &[
            (
                "main.jai",
                "#load \"alias.jai\"; main :: () { value: int = supplied; }",
            ),
            ("first.jai", "supplied :: 42;"),
            ("second.jai", "supplied :: \"changed\";"),
        ],
    );
    let alias = dir.path().join("alias.jai");
    std::os::unix::fs::symlink(dir.path().join("first.jai"), &alias).unwrap();
    let root = dir.path().join("main.jai");
    let uri = uri_of(&root);
    let mut session = lean_session();
    session
        .open(uri.clone(), 1, std::fs::read_to_string(root).unwrap())
        .unwrap();
    assert_eq!(errors(&session, &uri), 0);
    std::fs::remove_file(&alias).unwrap();
    session.file_changed(&alias, true);
    assert!(
        errors(&session, &uri) > 0,
        "removing a leaf alias removes the load"
    );
    std::os::unix::fs::symlink(dir.path().join("second.jai"), &alias).unwrap();
    session.file_changed(&alias, true);
    assert!(
        errors(&session, &uri) > 0,
        "new alias target must not reuse the first target"
    );
    std::fs::remove_file(&alias).unwrap();
    std::os::unix::fs::symlink(dir.path().join("first.jai"), &alias).unwrap();
    session.file_changed(&alias, true);
    assert_eq!(errors(&session, &uri), 0);
}

#[test]
fn silent_watched_loaded_file_edit_delete_and_recreate_are_rechecked() {
    let dir = project(
        "disk-freshness",
        &[
            (
                "main.jai",
                "#load \"dependency.jai\";\nmain :: () { value: int = supplied; }\n",
            ),
            ("dependency.jai", "supplied :: 42;\n"),
        ],
    );
    let root = dir.path().join("main.jai");
    let dependency = dir.path().join("dependency.jai");
    let uri = uri_of(&root);
    let mut session = lean_session();
    session
        .open(uri.clone(), 1, std::fs::read_to_string(&root).unwrap())
        .unwrap();
    assert_eq!(errors(&session, &uri), 0);
    let compiled = session.compile_counts().0;
    std::fs::write(&dependency, "supplied :: \"wrong type\";\n").unwrap();
    assert!(
        errors(&session, &uri) > 0,
        "edited loaded declaration must affect root diagnostics"
    );
    assert!(session.compile_counts().0 > compiled);
    std::fs::remove_file(&dependency).unwrap();
    assert!(errors(&session, &uri) > 0, "deleted load must be diagnosed");
    std::fs::write(&dependency, "supplied :: 42;\n").unwrap();
    assert!(
        errors(&session, &uri) == 0,
        "recreated dependency must repair diagnostics"
    );
}

#[test]
fn silent_a_failed_load_is_rechecked_when_the_file_appears() {
    let dir = project(
        "disk-created",
        &[(
            "main.jai",
            "#load \"later.jai\"; main :: () { supplied(); }\n",
        )],
    );
    let root = dir.path().join("main.jai");
    let dependency = dir.path().join("later.jai");
    let uri = uri_of(&root);
    let mut session = lean_session();
    session
        .open(uri.clone(), 1, std::fs::read_to_string(root).unwrap())
        .unwrap();
    assert!(errors(&session, &uri) > 0);
    std::fs::write(&dependency, "supplied :: () {}\n").unwrap();
    assert_eq!(errors(&session, &uri), 0);
}

#[test]
fn silent_watched_compile_time_data_reads_are_rechecked() {
    use test_support::lsp::session;
    let dir = project("disk-run-read", &[("data.txt", "one")]);
    let data = dir.path().join("data.txt");
    let source = format!(
        "#import \"Basic\"; #import \"File\"; DATA :: #run read_entire_file(\"{}\"); main :: () {{ #assert DATA == \"one\"; }}",
        jacket_config::slashed(&data)
    );
    let uri = uri_of(&dir.path().join("main.jai"));
    let mut session = session();
    session.open(uri.clone(), 1, source).unwrap();
    assert_eq!(errors(&session, &uri), 0);
    std::fs::write(&data, "two").unwrap();
    assert!(errors(&session, &uri) > 0);
}

#[cfg(unix)]
#[test]
fn silent_watched_leaf_symlink_delete_and_recreate_retire_target_facts() {
    let dir = project(
        "disk-symlink",
        &[
            (
                "main.jai",
                "#load \"alias.jai\"; main :: () { value: int = supplied; }",
            ),
            ("first.jai", "supplied :: 42;"),
            ("second.jai", "supplied :: \"changed\";"),
        ],
    );
    let alias = dir.path().join("alias.jai");
    std::os::unix::fs::symlink(dir.path().join("first.jai"), &alias).unwrap();
    let root = dir.path().join("main.jai");
    let uri = uri_of(&root);
    let mut session = lean_session();
    session
        .open(uri.clone(), 1, std::fs::read_to_string(root).unwrap())
        .unwrap();
    assert_eq!(errors(&session, &uri), 0);
    std::fs::remove_file(&alias).unwrap();
    assert!(
        errors(&session, &uri) > 0,
        "removing a leaf alias removes the load"
    );
    std::os::unix::fs::symlink(dir.path().join("second.jai"), &alias).unwrap();
    assert!(
        errors(&session, &uri) > 0,
        "new alias target must not reuse the first target"
    );
    std::fs::remove_file(&alias).unwrap();
    std::os::unix::fs::symlink(dir.path().join("first.jai"), &alias).unwrap();
    assert_eq!(errors(&session, &uri), 0);
}

#[test]
fn silent_metadata_preserved_source_rewrite_is_rechecked() {
    use jaic::sema::{FileSystem, NativeFs};
    let dir = project(
        "disk-preserved-metadata",
        &[
            (
                "main.jai",
                "#load \"dependency.jai\"; main :: () { value: int = supplied; }",
            ),
            ("dependency.jai", "supplied :: 42;\n"),
        ],
    );
    let dependency = dir.path().join("dependency.jai");
    let before = std::fs::metadata(&dependency).unwrap();
    let stamp = NativeFs.stamp(&dependency);
    let root = dir.path().join("main.jai");
    let uri = uri_of(&root);
    let mut session = lean_session();
    session
        .open(uri.clone(), 1, std::fs::read_to_string(root).unwrap())
        .unwrap();
    assert_eq!(errors(&session, &uri), 0);
    let changed = "supplied :: \"\";\n";
    assert_eq!(before.len() as usize, changed.len());
    std::fs::write(&dependency, changed).unwrap();
    std::fs::File::options()
        .write(true)
        .open(&dependency)
        .unwrap()
        .set_times(std::fs::FileTimes::new().set_modified(before.modified().unwrap()))
        .unwrap();
    assert_eq!(
        NativeFs.stamp(&dependency),
        stamp,
        "metadata fingerprints deliberately collide"
    );
    assert!(
        errors(&session, &uri) > 0,
        "observed read bytes changed without a notification or metadata change"
    );
}

#[test]
fn watched_dependency_change_proactively_publishes_to_push_clients() {
    use jai_language_server::{JsonSession, Limits};
    use serde_json::{Value, json};
    fn send(session: &mut JsonSession, value: Value) -> Vec<Value> {
        let output: Vec<serde_json::Value> = session
            .handle_json(&value.to_string())
            .unwrap()
            .iter()
            .map(|text| serde_json::from_str(text).unwrap())
            .collect();
        if value["method"] == "initialize"
            && output.first().is_some_and(|m| m.get("result").is_some())
        {
            assert!(
                session
                    .handle_json(r#"{"jsonrpc":"2.0","method":"initialized","params":{}}"#)
                    .unwrap()
                    .is_empty()
            );
        }
        output
    }
    let dir = project(
        "disk-push",
        &[
            (
                "main.jai",
                "#load \"dependency.jai\"; main :: () { supplied(); }",
            ),
            ("dependency.jai", "supplied :: () {}"),
        ],
    );
    let root = dir.path().join("main.jai");
    let dependency = dir.path().join("dependency.jai");
    let uri = uri_of(&root);
    let mut session =
        JsonSession::with_environment(Limits::default(), test_support::lsp::environment());
    send(
        &mut session,
        json!({"jsonrpc":"2.0","id":1,"method":"initialize","params":{"capabilities":{}}}),
    );
    send(
        &mut session,
        json!({"jsonrpc":"2.0","method":"textDocument/didOpen","params":{"textDocument":{"uri":uri.as_str(),"languageId":"jai","version":1,"text":std::fs::read_to_string(root).unwrap()}}}),
    );
    std::fs::write(&dependency, "replacement :: () {}").unwrap();
    let messages = send(
        &mut session,
        json!({"jsonrpc":"2.0","method":"workspace/didChangeWatchedFiles","params":{"changes":[{"uri":uri_of(&dependency).as_str(),"type":2}]}}),
    );
    assert!(
        messages.iter().any(
            |message| message["method"] == "textDocument/publishDiagnostics"
                && message["params"]["uri"] == uri.as_str()
                && message["params"]["diagnostics"]
                    .as_array()
                    .is_some_and(|d| !d.is_empty())
        ),
        "push clients need refreshed diagnostics without another edit/request"
    );
}

#[test]
fn silent_project_import_directory_change_recomputes_semantic_inputs() {
    let dir = project(
        "silent-import-config",
        &[
            ("jacket.toml", "[project]\nimport_path = [\"first\"]\n"),
            (
                "main.jai",
                "Lib :: #import \"Library\"; main :: () { value: int = Lib.supplied; }",
            ),
            ("first/Library/module.jai", "supplied :: 42;"),
            ("second/Library/module.jai", "supplied :: \"changed\";"),
        ],
    );
    let root = dir.path().join("main.jai");
    let uri = uri_of(&root);
    let mut session = lean_session();
    session.set_workspace_folders(vec![dir.path().to_path_buf()]);
    session
        .open(uri.clone(), 1, std::fs::read_to_string(root).unwrap())
        .unwrap();
    assert_eq!(errors(&session, &uri), 0);
    let stable = session.compile_counts();
    assert_eq!(errors(&session, &uri), 0);
    assert_eq!(
        session.compile_counts(),
        stable,
        "unchanged discovery proof permits reuse"
    );
    std::fs::write(
        dir.path().join("jacket.toml"),
        "[project]\nimport_path = [\"second\"]\n",
    )
    .unwrap();
    assert!(
        errors(&session, &uri) > 0,
        "silent config changes must move import selection"
    );
}

#[test]
fn silent_nearer_project_configuration_replaces_previous_inputs() {
    let dir = project(
        "silent-nearer-config",
        &[
            (
                "jacket.toml",
                "[project]\nentry = [\"sub/main.jai\"]\nimport_path = [\"first\"]\n",
            ),
            (
                "sub/main.jai",
                "Lib :: #import \"Library\"; main :: () { value: int = Lib.supplied; }",
            ),
            ("first/Library/module.jai", "supplied :: 42;"),
            ("second/Library/module.jai", "supplied :: \"changed\";"),
        ],
    );
    let root = dir.path().join("sub/main.jai");
    let uri = uri_of(&root);
    let mut session = lean_session();
    session.set_workspace_folders(vec![dir.path().to_path_buf()]);
    session
        .open(uri.clone(), 1, std::fs::read_to_string(root).unwrap())
        .unwrap();
    assert_eq!(errors(&session, &uri), 0);
    std::fs::write(
        dir.path().join("sub/jacket.toml"),
        "[project]\nentry = [\"main.jai\"]\nimport_path = [\"../second\"]\n",
    )
    .unwrap();
    assert!(
        errors(&session, &uri) > 0,
        "a formerly missing nearer config is a discovery input"
    );
}

#[test]
fn silent_project_entry_change_rechecks_the_unchanged_child() {
    let dir = project(
        "silent-entry-config",
        &[
            ("jacket.toml", "[project]\nentry = [\"first.jai\"]\n"),
            ("first.jai", "#load \"child.jai\"; supplied :: 42;"),
            (
                "second.jai",
                "#load \"child.jai\"; supplied :: \"changed\";",
            ),
            ("child.jai", "main :: () { value: int = supplied; }"),
        ],
    );
    let root = dir.path().join("child.jai");
    let uri = uri_of(&root);
    let mut session = lean_session();
    session.set_workspace_folders(vec![dir.path().to_path_buf()]);
    session
        .open(uri.clone(), 1, std::fs::read_to_string(root).unwrap())
        .unwrap();
    assert_eq!(errors(&session, &uri), 0);
    std::fs::write(
        dir.path().join("jacket.toml"),
        "[project]\nentry = [\"second.jai\"]\n",
    )
    .unwrap();
    assert!(
        errors(&session, &uri) > 0,
        "configured entry is part of the semantic context"
    );
}

#[test]
fn inconsistent_discovery_is_unavailable_until_the_next_request() {
    use jaic::sema::{FileSystem, NativeFs};
    struct ChangingConfig {
        path: std::path::PathBuf,
        once: std::cell::Cell<bool>,
    }
    impl FileSystem for ChangingConfig {
        fn is_native(&self) -> bool {
            true
        }
        fn read(&self, path: &std::path::Path) -> Option<Vec<u8>> {
            let bytes = NativeFs.read(path);
            if std::fs::canonicalize(path).is_ok_and(|path| path == self.path)
                && self.once.replace(false)
            {
                std::fs::write(path, "[project]\nimport_path = [\"second\"]\n").unwrap();
            }
            bytes
        }
        fn is_file(&self, path: &std::path::Path) -> bool {
            NativeFs.is_file(path)
        }
        fn is_dir(&self, path: &std::path::Path) -> bool {
            NativeFs.is_dir(path)
        }
        fn canonical(&self, path: &std::path::Path) -> std::path::PathBuf {
            NativeFs.canonical(path)
        }
        fn list_dir(&self, path: &std::path::Path) -> Vec<(String, bool)> {
            NativeFs.list_dir(path)
        }
        fn stamp(&self, path: &std::path::Path) -> Option<u64> {
            NativeFs.stamp(path)
        }
    }
    let dir = project(
        "mixed-discovery",
        &[
            ("jacket.toml", "[project]\nimport_path = [\"first\"]\n"),
            (
                "main.jai",
                "Lib :: #import \"Library\"; main :: () { value: int = Lib.supplied; }",
            ),
            ("first/Library/module.jai", "supplied :: 42;"),
            ("second/Library/module.jai", "supplied :: \"changed\";"),
        ],
    );
    let root = dir.path().join("main.jai");
    let uri = uri_of(&root);
    let mut session = Session::with_environment(
        jai_language_server::Limits::default(),
        jai_language_server::Environment {
            fs: std::rc::Rc::new(ChangingConfig {
                path: dir.path().join("jacket.toml").canonicalize().unwrap(),
                once: std::cell::Cell::new(true),
            }),
            services: None,
            options: Box::new(|_, _| {
                let mut options = jaic::sema::Options::host();
                options.runtime_support = false;
                options.stack_trace = false;
                options
            }),
        },
    );
    session.set_workspace_folders(vec![dir.path().to_path_buf()]);
    session
        .open(uri.clone(), 1, std::fs::read_to_string(root).unwrap())
        .unwrap();
    assert_eq!(
        session.diagnostics(&uri),
        Err(jai_language_server::Error::DiagnosticsUnavailable)
    );
    assert!(
        std::fs::read_to_string(dir.path().join("jacket.toml"))
            .unwrap()
            .contains("second"),
        "fixture must change config during an actual read"
    );
    assert_eq!(
        session.reference_record_counts(),
        (0, 0),
        "mixed discovery cannot retry internally and publish accepted facts"
    );
    assert!(
        errors(&session, &uri) > 0,
        "a new request can discover a fresh stable context"
    );
}

#[test]
fn silent_closed_dependency_rebuilds_retained_call_graphs() {
    use test_support::lsp::at_offset;
    let source = "#load \"dependency.jai\"; main :: () { selected(); }";
    let dir = project(
        "silent-calls",
        &[
            ("main.jai", source),
            (
                "dependency.jai",
                "first :: () {} second :: () {} selected :: first;",
            ),
        ],
    );
    let uri = uri_of(&dir.path().join("main.jai"));
    let mut session = lean_session();
    session.open(uri.clone(), 1, source.into()).unwrap();
    session.diagnostics(&uri).unwrap();
    let item = session
        .prepare_call_hierarchy(&uri, at_offset(source, "main ::", 1))
        .unwrap()
        .remove(0);
    assert_eq!(session.outgoing_calls(&item).unwrap()[0].item.name, "first");
    std::fs::write(
        dir.path().join("dependency.jai"),
        "first :: () {} second :: () {} selected :: second;",
    )
    .unwrap();
    let calls = session.outgoing_calls(&item).unwrap();
    assert_eq!(calls.len(), 1);
    assert_eq!(
        calls[0].item.name, "second",
        "retained caller facts depend on closed alias resolution"
    );
}

#[test]
fn silent_closed_caller_rebuilds_retained_polymorph_summaries() {
    use test_support::lsp::at_offset;
    let generic = "identity :: (x: $T) -> T { return x; }";
    let dir = project(
        "silent-polymorph",
        &[
            ("jacket.toml", "[project]\nentry=[\"main.jai\"]\n"),
            ("generic.jai", generic),
            (
                "main.jai",
                "#load \"generic.jai\"; main :: () { x:u8; identity(x); }",
            ),
        ],
    );
    let uri = uri_of(&dir.path().join("generic.jai"));
    let mut session = lean_session();
    session.set_workspace_folders(vec![dir.path().to_path_buf()]);
    session.open(uri.clone(), 1, generic.into()).unwrap();
    let at = at_offset(generic, "identity", 1);
    assert_eq!(session.polymorphs(&uri, at), ["T = u8"]);
    std::fs::write(
        dir.path().join("main.jai"),
        "#load \"generic.jai\"; main :: () { x:u16; identity(x); }",
    )
    .unwrap();
    assert_eq!(
        session.polymorphs(&uri, at),
        ["T = u16"],
        "same session revision does not prove closed caller inputs"
    );
}

#[cfg(unix)]
#[test]
fn silent_saved_config_alias_retarget_retires_its_previous_overlay_binding() {
    let dir = project(
        "saved-config-alias",
        &[
            (
                "a/main.jai",
                "L :: #import \"Library\"; main :: () { value: int = L.supplied; }",
            ),
            ("a/jacket.toml", "[project]\nimport_path=[\"vendor\"]\n"),
            ("a/vendor/Library/module.jai", "supplied :: \"wrong\";"),
            ("a/overlay/Library/module.jai", "supplied :: 42;"),
            ("b/jacket.toml", "[project]\nimport_path=[\"vendor\"]\n"),
        ],
    );
    let alias = dir.path().join("jacket.toml");
    std::os::unix::fs::symlink(dir.path().join("a/jacket.toml"), &alias).unwrap();
    let config = uri_of(&alias);
    let main = dir.path().join("a/main.jai");
    let uri = uri_of(&main);
    let mut session = lean_session();
    session
        .open(
            config.clone(),
            1,
            "[project]\nimport_path=[\"overlay\"]\n".into(),
        )
        .unwrap();
    session.save(&config).unwrap();
    session
        .open(uri.clone(), 1, std::fs::read_to_string(main).unwrap())
        .unwrap();
    assert_eq!(errors(&session, &uri), 0);
    std::fs::remove_file(&alias).unwrap();
    std::os::unix::fs::symlink(dir.path().join("b/jacket.toml"), &alias).unwrap();
    assert!(
        errors(&session, &uri) > 0,
        "saved config no longer overlays its former canonical target"
    );
}

#[test]
fn silent_enclosing_stdlib_appearance_invalidates_option_discovery() {
    use jai_language_server::{Environment, Limits};
    let dir = project(
        "options-stdlib-proof",
        &[("library/user/main.jai", "main :: () {}")],
    );
    let main = dir.path().join("library/user/main.jai");
    let default_stdlib = test_support::lsp::stdlib();
    let mut session = Session::with_environment(
        Limits::default(),
        Environment {
            fs: std::rc::Rc::new(jaic::sema::NativeFs),
            services: None,
            options: Box::new(move |fs, main| {
                let inputs = jaic::project::ProgramInputs::for_file(fs, main, &default_stdlib);
                let mut options = jaic::sema::Options::for_program(fs, main, &inputs);
                options.runtime_support = false;
                options.stack_trace = false;
                options
            }),
        },
    );
    let uri = uri_of(&main);
    session
        .open(uri.clone(), 1, "main :: () {}".into())
        .unwrap();
    assert_eq!(errors(&session, &uri), 0);
    let prelude = dir.path().join("library/prelude/module.jai");
    std::fs::create_dir_all(prelude.parent().unwrap()).unwrap();
    std::fs::write(&prelude, "#assert false;").unwrap();
    session.diagnostics(&uri).unwrap();
    let coverage = session.semantic_coverage(&uri).unwrap().unwrap();
    assert!(
        !coverage.program_complete && coverage.program_errors > 0,
        "a newly selected enclosing prelude must replace the old complete program"
    );
    assert!(
        coverage
            .analysis
            .import_dirs
            .unwrap()
            .iter()
            .any(|dir| dir.path == prelude.parent().unwrap().parent().unwrap())
    );
}
