//! Go to definition and document links on `#load` / `#import`, resolved as the compiler does.
use jai_language_server::{DocumentUri, Position, Session};
use test_support::lsp::{at_offset, file_uri, session, stdlib};

const MAIN: &str = r#"#import "Basic";
B :: #import "Basic";
#import "Mine";
#import "Both";
#import,file "lib/helper.jai";
#import,dir "pkg";
#load "part.jai";
#import "Missing";
printer :: B.print;
main :: () {
    B.print("%\n", deep());
    print("%\n", helper_value + pkg_value + part_value + mine_value + both_value);
}
"#;

fn open_all() -> Session {
    let mut s = session();
    let files = [
        ("main.jai", MAIN),
        (
            "modules/Mine.jai",
            "mine_value :: 1;\nusing Inner :: #import,file \"inner.jai\";\n",
        ),
        ("modules/inner.jai", "deep :: () -> int { return 7; }\n"),
        ("modules/Both.jai", "both_value :: 2;\n"),
        ("modules/Both/module.jai", "both_value :: 3;\n"),
        ("lib/helper.jai", "helper_value :: 4;\n"),
        ("pkg/module.jai", "pkg_value :: 5;\n"),
        ("part.jai", "part_value :: 6;\n"),
    ];
    for (path, text) in files {
        s.open(file_uri(&format!("/lsp-links-test/{path}")), 1, text.into())
            .unwrap();
    }
    s
}

fn target(s: &Session, position: Position) -> (String, Position) {
    let found = s
        .definition(&file_uri("/lsp-links-test/main.jai"), position)
        .unwrap();
    assert_eq!(
        found.len(),
        1,
        "at {position:?}: {found:?}; diagnostics: {:?}",
        s.diagnostics(&file_uri("/lsp-links-test/main.jai"))
    );
    (found[0].uri.clone(), found[0].range.start)
}

#[test]
fn import_and_load_strings_go_to_the_file_the_compiler_reads() {
    let s = open_all();
    let basic = DocumentUri::from_path(&stdlib().join("Basic/module.jai"))
        .unwrap()
        .as_str()
        .to_owned();
    let start = Position::default();
    for (marker, expected) in [
        ("\"Basic\";\nB", basic.clone()),
        (
            "\"Mine\"",
            file_uri("/lsp-links-test/modules/Mine.jai").as_str().into(),
        ),
        // `Name.jai` comes before `Name/module.jai`.
        (
            "\"Both\"",
            file_uri("/lsp-links-test/modules/Both.jai").as_str().into(),
        ),
        (
            "\"lib/helper.jai\"",
            file_uri("/lsp-links-test/lib/helper.jai").as_str().into(),
        ),
        (
            "\"pkg\"",
            file_uri("/lsp-links-test/pkg/module.jai").as_str().into(),
        ),
        (
            "\"part.jai\"",
            file_uri("/lsp-links-test/part.jai").as_str().into(),
        ),
    ] {
        assert_eq!(
            target(&s, at_offset(MAIN, marker, 2)),
            (expected, start),
            "{marker}"
        );
    }
    // The directive itself links too.
    assert_eq!(
        target(&s, at_offset(MAIN, "#load", 1)),
        (file_uri("/lsp-links-test/part.jai").as_str().into(), start)
    );
    // A module the compiler cannot find has no target.
    assert_eq!(
        s.definition(
            &file_uri("/lsp-links-test/main.jai"),
            at_offset(MAIN, "\"Missing\"", 2)
        )
        .unwrap(),
        [] as [jai_language_server::Location; 0]
    );
}

#[test]
fn document_links_span_the_strings() {
    let s = open_all();
    let links = s
        .document_links(&file_uri("/lsp-links-test/main.jai"))
        .unwrap();
    assert_eq!(links.len(), 7, "{links:?}");
    let (range, target) = &links[6];
    assert_eq!(range.start, at_offset(MAIN, "\"part.jai\"", 0));
    assert_eq!(range.end, at_offset(MAIN, "\"part.jai\"", 10));
    assert_eq!(target, file_uri("/lsp-links-test/part.jai").as_str());
}

#[test]
fn module_names_and_reexported_symbols_go_to_their_declarations() {
    let s = open_all();
    let diagnostics = s
        .diagnostics(&file_uri("/lsp-links-test/main.jai"))
        .unwrap();
    assert_eq!(diagnostics.len(), 1, "{diagnostics:?}");
    assert!(
        diagnostics[0]
            .message
            .contains("module `Missing` not found")
    );
    let basic = DocumentUri::from_path(&stdlib().join("Basic/module.jai"))
        .unwrap()
        .as_str()
        .to_owned();
    // `B` of `B.print`: the module's entry file.
    assert_eq!(
        target(&s, at_offset(MAIN, "B.print", 0)),
        (basic, Position::default())
    );
    // A resolved call navigates to its selected canonical overload.
    let print = s
        .definition(
            &file_uri("/lsp-links-test/main.jai"),
            at_offset(MAIN, "B.print(", 3),
        )
        .unwrap();
    assert!(
        print
            .iter()
            .all(|l| l.uri.contains("/stdlib/internal/basic/")),
        "{print:?}"
    );
    assert_eq!(print.len(), 1, "selected print overload: {print:?}");
    let overloads = s
        .definition(
            &file_uri("/lsp-links-test/main.jai"),
            at_offset(MAIN, "B.print;", 3),
        )
        .unwrap();
    assert_eq!(
        overloads.len(),
        2,
        "both reviewed alias overloads: {overloads:?}"
    );
    assert!(
        overloads
            .iter()
            .all(|l| l.uri.contains("/stdlib/internal/basic/"))
    );
    // `deep` reaches main through Mine's exported `using Inner :: #import`.
    assert_eq!(
        target(&s, at_offset(MAIN, "deep()", 1)),
        (
            file_uri("/lsp-links-test/modules/inner.jai")
                .as_str()
                .into(),
            Position::default()
        )
    );
    // Names of modules imported with a plain `#import` (not merged: each has its own scope).
    for (marker, file) in [
        ("mine_value", "modules/Mine.jai"),
        ("both_value", "modules/Both.jai"),
        ("helper_value", "lib/helper.jai"),
        ("pkg_value", "pkg/module.jai"),
        ("part_value", "part.jai"),
    ] {
        assert_eq!(
            target(&s, at_offset(MAIN, marker, 1)).0,
            file_uri(&format!("/lsp-links-test/{file}")).as_str(),
            "{marker}"
        );
    }
}

#[test]
fn document_links_share_import_options_within_one_request() {
    use std::cell::Cell;
    use std::rc::Rc;
    let calls = Rc::new(Cell::new(0));
    let mut environment = test_support::lsp::environment();
    let options = environment.options;
    let counted = calls.clone();
    environment.options = Box::new(move |fs, path| {
        counted.set(counted.get() + 1);
        options(fs, path)
    });
    let mut s = Session::with_environment(jai_language_server::Limits::default(), environment);
    let uri = file_uri("/link-options/main.jai");
    s.open(
        uri.clone(),
        1,
        "#import \"A\"; #import \"B\"; #load \"part.jai\";".into(),
    )
    .unwrap();
    for path in ["modules/A.jai", "modules/B.jai", "part.jai"] {
        s.open(file_uri(&format!("/link-options/{path}")), 1, String::new())
            .unwrap();
    }
    assert_eq!(s.document_links(&uri).unwrap().len(), 3);
    calls.set(0);
    assert_eq!(s.document_links(&uri).unwrap().len(), 3);
    assert_eq!(
        calls.get(),
        1,
        "one import-path lookup for the whole request"
    );
}

#[test]
fn relative_load_file_and_directory_links_need_no_module_options_or_compiler() {
    use jai_language_server::{Environment, Limits};
    use std::{cell::Cell, rc::Rc};
    let calls = Rc::new(Cell::new(0));
    let counted = calls.clone();
    let mut session = Session::with_environment(
        Limits::default(),
        Environment {
            fs: Rc::new(jaic::sema::NativeFs),
            services: None,
            options: Box::new(move |_, _| {
                counted.set(counted.get() + 1);
                jaic::sema::Options::host()
            }),
        },
    );
    let main = file_uri("/relative-link/main.jai");
    for path in ["part.jai", "helper.jai", "pkg/module.jai"] {
        session
            .open(
                file_uri(&format!("/relative-link/{path}")),
                1,
                "value :: 1;".into(),
            )
            .unwrap();
    }
    session
        .open(
            main.clone(),
            1,
            "#load \"part.jai\"; #import,file \"helper.jai\"; #import,dir \"pkg\";".into(),
        )
        .unwrap();
    let before = calls.get();
    let links = session.document_links(&main).unwrap();
    assert_eq!(links.len(), 3);
    assert_eq!(
        calls.get(),
        before,
        "relative imports never consult module search configuration"
    );
    assert_eq!(session.compile_counts().0, 0);
}

#[test]
fn named_module_links_do_not_fall_back_to_stdlib_after_failed_build_discovery() {
    let dir = test_support::lsp::project(
        "link-failed-producer",
        &[
            ("jacket.toml", "[project]\nentry = [\"build.jai\"]\n"),
            ("build.jai", "#run missing();"),
            ("src/main.jai", "#import \"Basic\"; main :: () {}"),
        ],
    );
    let (mut session, uri, _) = test_support::lsp::open_file(&dir, "src/main.jai");
    assert!(
        session.document_links(&uri).unwrap().is_empty(),
        "failed producer is not proof of a standalone module context"
    );
    std::fs::write(dir.join("build.jai"), "#run {};").unwrap();
    session.file_changed(&dir.join("build.jai"), false);
    let found = session.document_links(&uri).unwrap();
    assert_eq!(found.len(), 1);
    assert!(found[0].1.ends_with("/Basic/module.jai"));
}

#[test]
fn deferred_named_module_links_queue_discovery_without_foreground_probing() {
    let dir = test_support::lsp::project(
        "link-deferred-producer",
        &[
            ("jacket.toml", "[project]\nentry = [\"build.jai\"]\n"),
            ("build.jai", "#run {};"),
            ("src/main.jai", "#import \"Basic\"; main :: () {}"),
        ],
    );
    let (session, uri, _) = test_support::lsp::open_file(&dir, "src/main.jai");
    session.defer_compiles(true);
    let before = session.compile_counts().0;
    assert!(session.document_links(&uri).unwrap().is_empty());
    assert_eq!(session.compile_counts().0, before);
    assert!(
        session.compile_pending(),
        "unproven import context needs an ordinary background check"
    );
    assert_eq!(session.warm_pending(), Some(true));
    let warmed = session.compile_counts().0;
    let found = session.document_links(&uri).unwrap();
    assert_eq!(found.len(), 1);
    assert!(found[0].1.ends_with("/Basic/module.jai"));
    assert_eq!(session.compile_counts().0, warmed);
}

fn missing_literal_link() -> (test_support::TempDir, Session, DocumentUri, Position) {
    use jai_language_server::{Environment, Limits};
    use std::rc::Rc;
    let dir = test_support::TempDir::new("missing-literal-link");
    let path = dir.path().join("main.jai");
    let source = "#load \"missing.jai\"; main :: () {}";
    std::fs::write(&path, source).unwrap();
    let mut session = Session::with_environment(
        Limits::default(),
        Environment {
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
    let uri = DocumentUri::from_path(&path).unwrap();
    session.open(uri.clone(), 1, source.into()).unwrap();
    (dir, session, uri, at_offset(source, "missing.jai", 2))
}

#[test]
fn unresolved_literal_definition_does_not_try_an_ordinary_semantic_provider() {
    let (_dir, session, uri, position) = missing_literal_link();
    assert!(session.definition(&uri, position).unwrap().is_empty());
    assert_eq!(
        session.compile_counts().0,
        0,
        "recognized load resolution cannot fall through to a compile"
    );
}

#[test]
fn unresolved_literal_hover_does_not_try_expansion_or_format_providers() {
    let (_dir, session, uri, position) = missing_literal_link();
    assert!(session.hover(&uri, position).unwrap().is_none());
    assert_eq!(
        session.compile_counts().0,
        0,
        "recognized load string cannot fall through to a compile"
    );
}

#[test]
fn named_links_keep_silent_inputs_until_known_config_and_module_events() {
    let dir = test_support::lsp::project(
        "named-link-config-proof",
        &[
            (
                "jacket.toml",
                "[project]\nentry=[\"main.jai\"]\nimport_path=[\"first\"]\n",
            ),
            ("main.jai", "#import \"Owned_Link_Target\"; main :: () {}"),
            ("first/Owned_Link_Target.jai", "value :: 1;"),
            ("second/Owned_Link_Target.jai", "value :: 2;"),
        ],
    );
    let (mut session, uri, _) = test_support::lsp::open_file(&dir, "main.jai");
    let target = |folder: &str| {
        DocumentUri::from_path(&dir.join(folder).join("Owned_Link_Target.jai"))
            .unwrap()
            .as_str()
            .to_owned()
    };
    assert_eq!(session.document_links(&uri).unwrap()[0].1, target("first"));
    std::fs::write(
        dir.join("jacket.toml"),
        "[project]\nentry=[\"main.jai\"]\nimport_path=[\"second\"]\n",
    )
    .unwrap();
    assert_eq!(session.document_links(&uri).unwrap()[0].1, target("first"));
    session.file_changed(&dir.join("jacket.toml"), false);
    assert_eq!(session.document_links(&uri).unwrap()[0].1, target("second"));
    std::fs::remove_file(dir.join("second/Owned_Link_Target.jai")).unwrap();
    assert_eq!(session.document_links(&uri).unwrap()[0].1, target("second"));
    session.file_changed(&dir.join("second/Owned_Link_Target.jai"), true);
    assert!(session.document_links(&uri).unwrap().is_empty());
    std::fs::write(dir.join("second/Owned_Link_Target.jai"), "value :: 3;").unwrap();
    assert!(session.document_links(&uri).unwrap().is_empty());
    session.file_changed(&dir.join("second/Owned_Link_Target.jai"), true);
    assert_eq!(session.document_links(&uri).unwrap()[0].1, target("second"));
}
