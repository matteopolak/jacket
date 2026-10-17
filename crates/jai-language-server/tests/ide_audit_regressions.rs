//! Independent qualified/reexport/context cases with exact UTF16 definition oracles.
use jai_language_server::{
    DiagnosticSeverity, DocumentUri, Environment, Limits, Position, Range, Session, TextChange,
};
use jaic::sema::{Options, VirtualFs};
use std::{path::PathBuf, rc::Rc};

const VALUES: &str = "amount :: 23;\nstate: int;\nNumber :: int;\n#scope_file\nsecret :: 99;\n";
const FACADE: &str = "using,only(amount,state,Number) Values :: #import \"Values\";\n";
const MAIN: &str = "Facade :: #import \"Facade\";\nAlias :: Facade;\n#add_context marker: int;\nmain :: () {\n    note := \"🦊\"; first := Alias.amount;\n    Alias.state = first;\n    number: Alias.Number;\n    context.marker = number;\n    number = context.marker;\n}\n";

fn at(text: &str, needle: &str) -> Position {
    let prefix = &text[..text.find(needle).unwrap()];
    Position {
        line: prefix.matches('\n').count() as u32,
        character: prefix.rsplit('\n').next().unwrap().encode_utf16().count() as u32,
    }
}

fn session() -> Session {
    let mut fs = VirtualFs::default();
    fs.insert("/ide-audit/modules/Values/module.jai", VALUES.as_bytes());
    fs.insert("/ide-audit/modules/Facade/module.jai", FACADE.as_bytes());
    fs.insert(
        "/ide-audit/modules/Overloads/module.jai",
        "choose :: (x:int)->int {return x;}\nchoose :: (x:float)->float {return x;}\n".as_bytes(),
    );
    fs.insert("/ide-audit/prelude/module.jai", "".as_bytes());
    fs.insert(
        "/ide-audit/modules/Configured/module.jai",
        "#module_parameters (LIMIT: s64 = 5);\nvalue :: LIMIT;\n".as_bytes(),
    );
    Session::with_environment(
        Limits::default(),
        Environment {
            fs: Rc::new(fs),
            options: Box::new(|_| {
                let mut options = Options::host();
                options.runtime_support = false;
                options.stack_trace = false;
                options.preload = Some(PathBuf::from("/ide-audit/prelude/module.jai"));
                options.import_paths = vec![PathBuf::from("/ide-audit/modules")];
                options
            }),
        },
    )
}

fn check(s: &Session, uri: &DocumentUri, text: &str) {
    for (usage, definition) in [
        ("amount;", "amount ::"),
        ("state =", "state:"),
        ("Number;", "Number ::"),
    ] {
        let position = at(text, usage);
        let hover = s.hover(uri, position).unwrap().expect(usage);
        assert!(
            !hover.contents.value.contains("Source syntax declaration"),
            "{usage}: {}",
            hover.contents.value
        );
        let targets = s.definition(uri, position).unwrap();
        assert_eq!(targets.len(), 1, "{usage}: {targets:?}");
        assert_eq!(
            targets[0].uri,
            "file:///ide-audit/modules/Values/module.jai"
        );
        assert_eq!(targets[0].range.start, at(VALUES, definition));
    }
    for usage in ["marker =", "marker;"] {
        let targets = s.definition(uri, at(text, usage)).unwrap();
        assert_eq!(targets.len(), 1, "{usage}: {targets:?}");
        assert_eq!(targets[0].uri, uri.as_str());
        assert_eq!(targets[0].range.start, at(text, "marker: int"));
        let hover = s.hover(uri, at(text, usage)).unwrap().expect(usage);
        assert!(!hover.contents.value.contains("Source syntax declaration"));
    }
}

#[test]
fn qualified_reexports_and_context_keep_exact_locations_after_utf16_edits() {
    let uri = DocumentUri::parse("file:///ide-audit/main.jai").unwrap();
    let mut session = session();
    session.open(uri.clone(), 1, MAIN.into()).unwrap();
    let errors = session.diagnostics(&uri).unwrap();
    assert!(
        errors
            .iter()
            .all(|d| d.severity != DiagnosticSeverity::Error),
        "{errors:?}"
    );
    check(&session, &uri, MAIN);
    let insertion = at(MAIN, "first :=");
    session
        .change(
            &uri,
            2,
            &[TextChange {
                range: Some(Range {
                    start: insertion,
                    end: insertion,
                }),
                range_length: Some(0),
                text: "extra := \"🎯\"; ".into(),
            }],
        )
        .unwrap();
    let changed = MAIN.replace("first :=", "extra := \"🎯\"; first :=");
    check(&session, &uri, &changed);
    session
        .change(
            &uri,
            3,
            &[TextChange {
                range: None,
                range_length: None,
                text: MAIN.into(),
            }],
        )
        .unwrap();
    check(&session, &uri, MAIN);
}

#[test]
fn nested_initializer_targets_and_contextual_enums_keep_exact_field_definitions() {
    const SOURCE: &str = "Record :: struct { kind: enum { FIRST; SECOND; } detail: struct { count:int; } }\nmain :: () { value:=Record.{kind=.SECOND, detail.count=13}; observed:=value.detail.count; }\n";
    let uri = DocumentUri::parse("file:///ide-audit/main.jai").unwrap();
    let mut s = session();
    s.open(uri.clone(), 1, SOURCE.into()).unwrap();
    let diagnostics = s.diagnostics(&uri).unwrap();
    assert!(
        diagnostics
            .iter()
            .all(|d| d.severity != DiagnosticSeverity::Error),
        "{diagnostics:?}"
    );
    for (usage, declaration) in [
        ("SECOND,", "SECOND;"),
        ("detail.count=", "detail: struct"),
        ("count=13", "count:int"),
    ] {
        let position = at(SOURCE, usage);
        let hover = s.hover(&uri, position).unwrap().expect(usage);
        assert!(
            !hover.contents.value.contains("Source syntax declaration"),
            "{usage}: {hover:?}"
        );
        let definitions = s.definition(&uri, position).unwrap();
        assert_eq!(definitions.len(), 1, "{usage}: {definitions:?}");
        assert_eq!(definitions[0].uri, uri.as_str());
        assert_eq!(
            definitions[0].range.start,
            at(SOURCE, declaration),
            "{usage}"
        );
    }
}

#[test]
fn builtin_type_uses_hover_without_fabricated_source_definitions() {
    let uri = DocumentUri::parse("file:///ide-audit/main.jai").unwrap();
    let text = "value: int;\nmain :: () { other: s32; }\n";
    let mut session = session();
    session.open(uri.clone(), 1, text.into()).unwrap();
    for name in ["int;", "s32;"] {
        let hover = session.hover(&uri, at(text, name)).unwrap().expect(name);
        assert!(!hover.contents.value.contains("Source syntax declaration"));
        assert!(session.definition(&uri, at(text, name)).unwrap().is_empty());
    }
}

#[test]
fn static_reexport_filter_names_navigate_to_visible_original_declarations() {
    let uri = DocumentUri::parse("file:///ide-audit/main.jai").unwrap();
    let facade = DocumentUri::parse("file:///ide-audit/modules/Facade/module.jai").unwrap();
    let mut s = session();
    s.open(uri, 1, MAIN.into()).unwrap();
    s.open(facade.clone(), 1, FACADE.into()).unwrap();
    for (usage, declaration) in [
        ("amount,state", "amount ::"),
        ("state,Number", "state:"),
        ("Number)", "Number ::"),
    ] {
        let p = at(FACADE, usage);
        let hover = s.hover(&facade, p).unwrap().expect(usage);
        assert!(!hover.contents.value.contains("Source syntax declaration"));
        let targets = s.definition(&facade, p).unwrap();
        assert_eq!(targets.len(), 1, "{usage}: {targets:?}");
        assert_eq!(
            targets[0].uri,
            "file:///ide-audit/modules/Values/module.jai"
        );
        assert_eq!(targets[0].range.start, at(VALUES, declaration));
    }
}

#[test]
fn selector_metadata_retains_overloads_and_does_not_expose_private_or_absent_names() {
    let uri = DocumentUri::parse("file:///ide-audit/main.jai").unwrap();
    let text = "using,only(choose) M :: #import \"Overloads\";\nmain :: () {value:=choose(1);}\n";
    let mut s = session();
    s.open(uri.clone(), 1, text.into()).unwrap();
    let p = at(text, "choose)");
    assert!(s.hover(&uri, p).unwrap().is_some());
    let targets = s.definition(&uri, p).unwrap();
    assert_eq!(targets.len(), 2, "{targets:?}");
    assert!(targets.iter().all(
        |t| t.uri == "file:///ide-audit/modules/Overloads/module.jai"
            && t.range.start.character == 0
    ));
    assert_eq!(
        targets
            .iter()
            .map(|t| t.range.start.line)
            .collect::<Vec<_>>(),
        vec![0, 1]
    );
    let text = "using,only(secret,absent) M :: #import \"Values\";\nmain :: () {}\n";
    s.change(
        &uri,
        2,
        &[TextChange {
            range: None,
            range_length: None,
            text: text.into(),
        }],
    )
    .unwrap();
    assert!(
        s.diagnostics(&uri)
            .unwrap()
            .iter()
            .all(|d| d.severity != DiagnosticSeverity::Error)
    );
    for name in ["secret,", "absent)"] {
        assert!(s.definition(&uri, at(text, name)).unwrap().is_empty());
    }
}

#[test]
fn static_aggregate_filters_use_actual_field_identity() {
    let uri = DocumentUri::parse("file:///ide-audit/main.jai").unwrap();
    let text = "R :: struct {field:int;}\nmain :: () {r:R; using,only(field) r; field=3;}\n";
    let mut s = session();
    s.open(uri.clone(), 1, text.into()).unwrap();
    let targets = s.definition(&uri, at(text, "field)")).unwrap();
    assert_eq!(targets.len(), 1, "{targets:?}");
    assert_eq!(targets[0].uri, uri.as_str());
    assert_eq!(targets[0].range.start, at(text, "field:int"));
    assert!(s.hover(&uri, at(text, "field)")).unwrap().is_some());
}

#[test]
fn aggregate_using_targets_keep_the_original_local_and_global_type_and_identity() {
    let uri = DocumentUri::parse("file:///ide-audit/main.jai").unwrap();
    let text = "R :: struct {field:int;}\nglobal:R;\nmain :: () {local:R; using,only(field) local; using,except(absent) global;}\n";
    let mut s = session();
    s.open(uri.clone(), 1, text.into()).unwrap();
    for (usage, decl) in [("local;", "local:R"), ("global;", "global:R")] {
        let hover = s.hover(&uri, at(text, usage)).unwrap().expect(usage);
        assert!(
            !hover.contents.value.contains("Source syntax declaration"),
            "{usage}: {}",
            hover.contents.value
        );
        assert!(
            hover.contents.value.contains("R"),
            "{}",
            hover.contents.value
        );
        let targets = s.definition(&uri, at(text, usage)).unwrap();
        assert_eq!(targets.len(), 1, "{usage}: {targets:?}");
        assert_eq!(targets[0].uri, uri.as_str());
        assert_eq!(targets[0].range.start, at(text, decl));
    }
}

#[test]
fn named_module_arguments_navigate_to_real_parameter_declarations_on_cached_instances() {
    let uri = DocumentUri::parse("file:///ide-audit/main.jai").unwrap();
    let text = "First :: #import \"Configured\"(LIMIT=12);\nAgain :: #import \"Configured\"(LIMIT=12);\nOther :: #import \"Configured\"(LIMIT=9);\nmain :: () {}\n";
    let mut s = session();
    s.open(uri.clone(), 1, text.into()).unwrap();
    assert!(
        s.diagnostics(&uri)
            .unwrap()
            .iter()
            .all(|d| d.severity != DiagnosticSeverity::Error)
    );
    for line in 0..3 {
        let source_line = text.lines().nth(line as usize).unwrap();
        let p = Position {
            line,
            character: source_line.find("LIMIT=").unwrap() as u32,
        };
        let hover = s.hover(&uri, p).unwrap().expect("named module argument");
        assert!(
            hover.contents.value.contains("s64"),
            "{}",
            hover.contents.value
        );
        let targets = s.definition(&uri, p).unwrap();
        assert_eq!(targets.len(), 1, "{targets:?}");
        assert_eq!(
            targets[0].uri,
            "file:///ide-audit/modules/Configured/module.jai"
        );
        assert_eq!(
            targets[0].range.start,
            at("#module_parameters (LIMIT: s64 = 5);", "LIMIT:")
        );
    }
}

#[test]
fn invalid_module_argument_labels_keep_failed_context_without_fabricated_targets() {
    let uri = DocumentUri::parse("file:///ide-audit/main.jai").unwrap();
    let text = "Bad :: #import \"Configured\"(ABSENT=12);\nmain :: () {value:=Bad.value;}\n";
    let mut s = session();
    s.open(uri.clone(), 1, text.into()).unwrap();
    // The binding diagnostic may be attached to the module parameter header,
    // rather than this caller file. Empty local diagnostics cannot imply success.
    let coverage = s
        .semantic_coverage(&uri)
        .unwrap()
        .expect("analysis context");
    assert!(
        coverage.program_errors > 0 && !coverage.program_complete,
        "{coverage:?}"
    );
    assert!(s.definition(&uri, at(text, "ABSENT=")).unwrap().is_empty());
}

#[test]
fn malformed_c_variadic_call_diagnoses_and_valid_edit_restores_navigation() {
    let uri = DocumentUri::parse("file:///ide-audit/main.jai").unwrap();
    let bad = "broken :: (items: ..int, tail:int) #c_call {}\nmain :: () {broken(1,tail=2);}\n";
    let mut s = session();
    s.open(uri.clone(), 1, bad.into()).unwrap();
    let diagnostics = s.diagnostics(&uri).unwrap();
    assert!(
        diagnostics
            .iter()
            .any(|d| d.severity == DiagnosticSeverity::Error
                && d.message.contains("variadic")
                && d.message.contains("last")),
        "{diagnostics:?}"
    );
    // Querying either a valid binding or the failed call must never panic.
    let _ = s.hover(&uri, at(bad, "broken(1")).unwrap();
    let _ = s.definition(&uri, at(bad, "broken(1")).unwrap();
    let good = "sum :: (items: ..int, tail:int=4) -> int {value:=tail; for items value+=it; return value;}\nmain :: () {value:=sum(1,tail=2);}\n";
    s.change(
        &uri,
        2,
        &[TextChange {
            range: None,
            range_length: None,
            text: good.into(),
        }],
    )
    .unwrap();
    assert!(
        s.diagnostics(&uri)
            .unwrap()
            .iter()
            .all(|d| d.severity != DiagnosticSeverity::Error)
    );
    let targets = s.definition(&uri, at(good, "sum(1")).unwrap();
    assert_eq!(targets.len(), 1, "{targets:?}");
    assert_eq!(targets[0].range.start, at(good, "sum ::"));
    assert!(s.hover(&uri, at(good, "sum(1")).unwrap().is_some());
}

#[test]
fn configured_bootstrap_type_uses_keep_semantic_hovers() {
    let uri = DocumentUri::parse("file:///ide-audit/main.jai").unwrap();
    let text = "value: int;\nmain :: () { other: s32; }\n";
    let stdlib = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../stdlib")
        .canonicalize()
        .unwrap();
    let mut session = Session::with_environment(
        Limits::default(),
        Environment {
            fs: Rc::new(jaic::sema::NativeFs),
            options: Box::new(move |_| {
                let mut options = Options::host();
                options.stack_trace = true;
                options.preload = Some(stdlib.join(jaic::PRELOAD_ENTRY));
                options.import_paths = vec![stdlib.clone()];
                options
            }),
        },
    );
    session.open(uri.clone(), 1, text.into()).unwrap();
    assert!(
        session
            .diagnostics(&uri)
            .unwrap()
            .iter()
            .all(|d| d.severity != DiagnosticSeverity::Error)
    );
    for name in ["int;", "s32;"] {
        let hover = session.hover(&uri, at(text, name)).unwrap().expect(name);
        assert!(!hover.contents.value.contains("Source syntax declaration"));
    }
}

#[test]
fn production_basic_temp_and_exit_remain_typed_and_navigate() {
    let uri = DocumentUri::parse("file:///ide-audit/main.jai").unwrap();
    let text = "using #import \"Basic\";\nBasic :: #import \"Basic\";\nmain :: () { allocator := Basic.temp; exit(1); }\n";
    let stdlib = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../stdlib")
        .canonicalize()
        .unwrap();
    let mut s = Session::with_environment(
        Limits::default(),
        Environment {
            fs: Rc::new(jaic::sema::NativeFs),
            options: Box::new(move |_| {
                let mut options = Options::host();
                options.preload = Some(stdlib.join(jaic::PRELOAD_ENTRY));
                options.import_paths = vec![stdlib.clone()];
                options
            }),
        },
    );
    s.open(uri.clone(), 1, text.into()).unwrap();
    assert!(
        s.diagnostics(&uri)
            .unwrap()
            .iter()
            .all(|d| d.severity != DiagnosticSeverity::Error)
    );
    for (name, expected) in [("temp;", "Allocator"), ("exit(1)", "s32")] {
        let hover = s.hover(&uri, at(text, name)).unwrap().expect(name);
        assert!(
            hover.contents.value.contains(expected),
            "{}",
            hover.contents.value
        );
        let targets = s.definition(&uri, at(text, name)).unwrap();
        assert!(!targets.is_empty(), "{name}");
        assert!(
            targets
                .iter()
                .all(|t| t.uri.starts_with("file://") && t.uri.contains("/stdlib/"))
        );
    }
}

#[test]
fn open_project_modules_do_not_erase_root_type_facts() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/lsp/audit/values")
        .canonicalize()
        .unwrap();
    let stdlib = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../stdlib")
        .canonicalize()
        .unwrap();
    let mut session = Session::with_environment(
        Limits::default(),
        Environment {
            fs: Rc::new(jaic::sema::NativeFs),
            options: Box::new(move |_| {
                let mut options = Options::host();
                options.stack_trace = true;
                options.preload = Some(stdlib.join(jaic::PRELOAD_ENTRY));
                options.import_paths = vec![stdlib.clone()];
                options
            }),
        },
    );
    session.set_workspace_folders(vec![root.clone()]);
    let uri = DocumentUri::from_path(&root.join("main.jai")).unwrap();
    let text = std::fs::read_to_string(root.join("main.jai")).unwrap();
    session.open(uri.clone(), 1, text.clone()).unwrap();
    let module = root.join("modules/Probe/module.jai");
    session
        .open(
            DocumentUri::from_path(&module).unwrap(),
            1,
            std::fs::read_to_string(&module).unwrap(),
        )
        .unwrap();
    let errors = session.diagnostics(&uri).unwrap();
    assert!(
        errors
            .iter()
            .all(|d| d.severity != DiagnosticSeverity::Error),
        "{errors:?}"
    );
    for token in jaic::lexer::lex(jaic::source::FileId(0), &text).unwrap() {
        if matches!(
            token.tok,
            jaic::lexer::Tok::Ident(_) | jaic::lexer::Tok::Directive(_)
        ) {
            let prefix = &text[..token.span.start as usize];
            let p = Position {
                line: prefix.matches('\n').count() as u32,
                character: prefix.rsplit('\n').next().unwrap().encode_utf16().count() as u32,
            };
            let hover = session.hover(&uri, p).unwrap();
            if matches!(token.tok,jaic::lexer::Tok::Ident(name) if name.as_str()=="int") {
                assert!(
                    hover.is_some_and(|h| !h.contents.value.contains("Source syntax declaration")),
                    "first request at {p:?}"
                );
            }
            let _ = session.definition(&uri, p).unwrap();
        }
    }
    for name in ["int;", "context.audit_marker ="] {
        let hover = session.hover(&uri, at(&text, name)).unwrap().expect(name);
        assert!(!hover.contents.value.contains("Source syntax declaration"));
    }
}

#[test]
fn an_open_unloaded_source_file_has_semantic_facts_in_its_own_file_context() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/lsp/audit/orphans")
        .canonicalize()
        .unwrap();
    let stdlib = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../stdlib")
        .canonicalize()
        .unwrap();
    let mut s = Session::with_environment(
        Limits::default(),
        Environment {
            fs: Rc::new(jaic::sema::NativeFs),
            options: Box::new(move |_| {
                let mut options = Options::host();
                options.preload = Some(stdlib.join(jaic::PRELOAD_ENTRY));
                options.import_paths = vec![stdlib.clone()];
                options
            }),
        },
    );
    s.set_workspace_folders(vec![root.clone()]);
    for file in ["main.jai", "extras.jai"] {
        s.open(
            DocumentUri::from_path(&root.join(file)).unwrap(),
            1,
            std::fs::read_to_string(root.join(file)).unwrap(),
        )
        .unwrap();
    }
    let uri = DocumentUri::from_path(&root.join("extras.jai")).unwrap();
    let text = std::fs::read_to_string(root.join("extras.jai")).unwrap();
    assert!(
        s.diagnostics(&uri)
            .unwrap()
            .iter()
            .all(|d| d.severity != DiagnosticSeverity::Error)
    );
    for (name, decl) in [
        ("Value.{", "Value ::"),
        ("number +", "number:"),
        ("hidden;", "hidden ::"),
    ] {
        let hover = s.hover(&uri, at(&text, name)).unwrap().expect(name);
        assert!(
            !hover.contents.value.contains("Source syntax declaration"),
            "{name}"
        );
        let targets = s.definition(&uri, at(&text, name)).unwrap();
        assert_eq!(targets.len(), 1, "{name}: {targets:?}");
        assert_eq!(targets[0].range.start, at(&text, decl));
        assert_eq!(targets[0].uri, uri.as_str());
    }
}
