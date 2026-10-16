//! Original failed-discovery reducers; no corpus or stdlib implementation inputs.
use jaic::sema::{Compiler, FileSystem, Options, VirtualFs, ide::IdeFacts};
use std::{
    cell::{Cell, RefCell},
    path::{Path, PathBuf},
    rc::Rc,
};

#[derive(Default)]
struct CountingFs {
    fs: RefCell<VirtualFs>,
    listings: Cell<usize>,
}
impl CountingFs {
    fn insert(&self, path: &str, text: &str) {
        self.fs.borrow_mut().insert(path, text.as_bytes());
    }
}
impl FileSystem for CountingFs {
    fn read(&self, path: &Path) -> Option<Vec<u8>> {
        self.fs.borrow().read(path)
    }
    fn is_file(&self, path: &Path) -> bool {
        self.fs.borrow().is_file(path)
    }
    fn is_dir(&self, path: &Path) -> bool {
        self.fs.borrow().is_dir(path)
    }
    fn list_dir(&self, path: &Path) -> Vec<(String, bool)> {
        self.listings.set(self.listings.get() + 1);
        self.fs.borrow().list_dir(path)
    }
}
fn compiler(source: &str) -> (Compiler, Rc<CountingFs>, jaic::sema::ScopeId, usize) {
    let fs = Rc::new(CountingFs::default());
    fs.insert("/discovery/main.jai", source);
    fs.insert(
        "/discovery/modules/Available/module.jai",
        "published :: 42;",
    );
    fs.insert(
        "/discovery/modules/Configured/module.jai",
        "#module_parameters(ENABLED: bool); published :: ENABLED;",
    );
    let mut options = Options::host();
    options.runtime_support = false;
    options.stack_trace = false;
    options.import_paths = vec!["/discovery/modules".into()];
    let mut c = Compiler::new(options, fs.clone());
    assert!(c.compile_program(Path::new("/discovery/main.jai")).is_err());
    let (scope, index) = c
        .scopes
        .iter()
        .enumerate()
        .find_map(|(s, scope)| {
            scope.imports.iter().enumerate().find_map(|(i, entry)| {
                matches!(&entry.import.source, jaic::ast::ImportSource::Module(name)
                if &**name == "Unavailable" || &**name == "Configured")
                .then_some((jaic::sema::ScopeId(s as u32), i))
            })
        })
        .unwrap();
    c.ide = Some(Box::new(IdeFacts::new(vec!["/discovery/".into()])));
    fs.listings.set(0);
    (c, fs, scope, index)
}
const SOURCE: &str = "#import \"Unavailable\"; #import \"Available\"; healthy :: () -> s64 { return published; } independent :: () { independent_missing; } trigger :: () -> s64 { return 7; } main :: () {}";

#[test]
fn unchanged_missing_import_reuses_discovery_and_keeps_real_errors() {
    let (mut c, fs, scope, index) = compiler(SOURCE);
    let first = c.import_module(scope, index).unwrap_err();
    let first_detail = format!("{first:?}");
    let listings = fs.listings.get();
    assert!(
        listings > 0,
        "first discovery must really inspect the filesystem"
    );
    for _ in 0..32 {
        let error = c.import_module(scope, index).unwrap_err();
        assert_eq!(
            format!("{error:?}"),
            first_detail,
            "preserve spans, notes, fixes and help"
        );
    }
    assert_eq!(
        fs.listings.get(),
        listings,
        "unchanged failed import must not repeat filesystem discovery"
    );
    let errors = c.ide_check_all();
    assert!(
        errors.iter().any(|e| e.message.contains("Unavailable")),
        "{errors:?}"
    );
    assert!(
        errors
            .iter()
            .any(|e| e.message.contains("independent_missing")),
        "{errors:?}"
    );
    let healthy = c
        .program
        .func_names
        .iter()
        .position(|n| n == "healthy")
        .unwrap();
    assert!(
        c.program.funcs[healthy].is_some(),
        "resolved imported declarations remain checked"
    );
}

#[test]
fn changed_import_directories_retry_real_discovery() {
    let (mut c, fs, scope, index) = compiler(SOURCE);
    assert!(c.import_module(scope, index).is_err());
    fs.insert("/alternate/Unavailable/module.jai", "new_binding :: 19;");
    c.options.import_paths.push(PathBuf::from("/alternate"));
    let module = c.import_module(scope, index).unwrap().unwrap();
    assert!(c.modules[module.0 as usize].files.len() == 1);
}

#[test]
fn actual_compile_time_execution_invalidates_missing_discovery() {
    let (mut c, fs, scope, index) = compiler(SOURCE);
    assert!(c.import_module(scope, index).is_err());
    fs.insert(
        "/discovery/modules/Unavailable/module.jai",
        "new_binding :: 19;",
    );
    let text = "value :: #run trigger();";
    let file = c
        .sources
        .add("/discovery/execution-probe.jai".into(), text.into());
    let parsed = jaic::parser::parse_file(file, text).unwrap();
    let jaic::ast::StmtKind::Decl(decl) = &parsed.stmts[0].kind else {
        panic!("declaration");
    };
    let value = c
        .eval_const(scope, decl.value.as_ref().unwrap(), None)
        .unwrap();
    assert!(matches!(
        value.const_value(),
        Some(jaic::sema::value::Value::Int(7))
    ));
    assert!(
        c.import_module(scope, index).unwrap().is_some(),
        "actual execution must permit filesystem changes"
    );
}

#[test]
fn argument_binding_progress_is_resolved_before_missing_discovery_reuse() {
    let (mut c, _, scope, index) =
        compiler("#import \"Configured\"(ENABLED = setting); main :: () {}");
    assert!(
        c.import_module(scope, index)
            .unwrap_err()
            .message
            .contains("setting")
    );
    c.add_source(&jaic::sema::ProgramSource::String(
        "setting :: true;".into(),
    ))
    .unwrap();
    assert!(c.import_module(scope, index).unwrap().is_some());
}

#[test]
fn strict_resolution_does_not_reuse_editor_discovery_memo() {
    let (mut c, fs, scope, index) = compiler(SOURCE);
    assert!(c.import_module(scope, index).is_err());
    c.ide = None;
    let before = fs.listings.get();
    assert!(c.import_module(scope, index).is_err());
    let after = fs.listings.get();
    assert!(after > before);
    assert!(c.import_module(scope, index).is_err());
    assert!(fs.listings.get() > after);
}

#[test]
fn incremental_and_fresh_sessions_keep_imported_navigation_and_retry_discovery() {
    use jai_language_server::{
        DocumentUri, Environment, Limits, Position, Range, Session, TextChange,
    };
    let fs = Rc::new(CountingFs::default());
    let text = "// 🧭\n#import \"Unavailable\";\n#import \"Available\";\nmain :: () { value := published; }\n";
    fs.insert("/discovery/main.jai", text);
    fs.insert(
        "/discovery/modules/Available/module.jai",
        "published :: 42;",
    );
    let make_session = || {
        Session::with_environment(
            Limits::default(),
            Environment {
                fs: fs.clone(),
                options: Box::new(|_| {
                    let mut options = Options::host();
                    options.runtime_support = false;
                    options.stack_trace = false;
                    options.import_paths = vec!["/discovery/modules".into()];
                    options
                }),
            },
        )
    };
    let uri = DocumentUri::parse("file:///discovery/main.jai").unwrap();
    let pos = Position {
        line: 3,
        character: text.lines().nth(3).unwrap().find("published").unwrap() as u32,
    };
    let mut session = make_session();
    session.open(uri.clone(), 1, text.into()).unwrap();
    assert!(session.diagnostics(&uri).unwrap().iter().any(|e| e.severity
        == jai_language_server::DiagnosticSeverity::Error
        && e.message.contains("Unavailable")));
    assert!(session.hover(&uri, pos).unwrap().is_some());
    let definitions = session.definition(&uri, pos).unwrap();
    assert_eq!(definitions.len(), 1, "{definitions:?}");
    assert!(
        definitions[0].uri.ends_with("/Available/module.jai"),
        "{definitions:?}"
    );
    assert!(
        session
            .completion(&uri, pos)
            .unwrap()
            .items
            .iter()
            .any(|item| item.label == "published")
    );

    fs.insert("/discovery/modules/Unavailable/module.jai", "other :: 7;");
    session.file_changed(Path::new("/discovery/modules/Unavailable/module.jai"), true);
    let end = Position {
        line: 0,
        character: 5,
    }; // UTF-16 boundary after the emoji.
    session
        .change(
            &uri,
            2,
            &[TextChange {
                range: Some(Range {
                    start: end,
                    end,
                }),
                range_length: Some(0),
                text: " ".into(),
            }],
        )
        .unwrap();
    assert!(
        !session.diagnostics(&uri).unwrap().iter().any(|e| e.severity
            == jai_language_server::DiagnosticSeverity::Error
            && e.message.contains("Unavailable"))
    );
    assert_eq!(session.definition(&uri, pos).unwrap(), definitions);
    let mut fresh = make_session();
    fresh
        .open(uri.clone(), 1, text.replacen("🧭", "🧭 ", 1))
        .unwrap();
    assert!(!fresh.diagnostics(&uri).unwrap().iter().any(|e| e.severity
        == jai_language_server::DiagnosticSeverity::Error
        && e.message.contains("Unavailable")));
    assert_eq!(fresh.definition(&uri, pos).unwrap(), definitions);
}
