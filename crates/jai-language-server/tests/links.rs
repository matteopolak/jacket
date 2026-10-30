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
