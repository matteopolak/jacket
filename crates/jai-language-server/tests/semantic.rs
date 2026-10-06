//! Type-checked hover and completion against the repository's stdlib.
use jai_language_server::{
    CompletionKind, DocumentUri, Environment, Limits, Position, Session, TextChange,
};
use std::path::PathBuf;

fn session() -> Session {
    let stdlib = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../stdlib");
    Session::with_environment(
        Limits::default(),
        Environment {
            fs: std::rc::Rc::new(jaic::sema::NativeFs),
            options: Box::new(move |_| {
                let mut options = jaic::sema::Options::host();
                options.import_paths = vec![stdlib.clone()];
                options.preload = Some(stdlib.join("prelude/module.jai"));
                options
            }),
        },
    )
}

fn uri() -> DocumentUri {
    DocumentUri::parse("file:///lsp-semantic-test/main.jai").unwrap()
}

/// Position just after the `n`th (0-based) occurrence of `marker`, minus `back` characters.
fn after(text: &str, marker: &str, n: usize, back: usize) -> Position {
    let byte = text.match_indices(marker).nth(n).unwrap().0 + marker.len() - back;
    let prefix = &text[..byte];
    Position {
        line: prefix.matches('\n').count() as u32,
        character: prefix.rsplit('\n').next().unwrap().len() as u32,
    }
}

fn labels(session: &Session, at: Position) -> Vec<(String, CompletionKind, String)> {
    session
        .completion(&uri(), at)
        .unwrap()
        .items
        .into_iter()
        .map(|i| (i.label, i.kind, i.detail))
        .collect()
}

fn hover(session: &Session, at: Position) -> String {
    session
        .hover(&uri(), at)
        .unwrap()
        .expect("hover")
        .contents
        .value
}

const PROGRAM: &str = r#"#import "Basic";
Thing :: struct { alpha: int; beta: float; label: string; }
Mode :: enum { IDLE; RUNNING; }
helper :: (t: *Thing) -> int {
    doubled := t.alpha * 2;
    return doubled;
}
main :: () {
    counter := 5;
    thing: Thing;
    total := counter + helper(*thing);
    print("%\n", total);
}
"#;

#[test]
fn hover_shows_types_of_locals_procedures_structs_and_members() {
    let mut s = session();
    s.open(uri(), 1, PROGRAM.into()).unwrap();
    assert_eq!(
        hover(&s, after(PROGRAM, "counter + ", 0, 4)),
        "counter: s64"
    );
    assert_eq!(
        hover(&s, after(PROGRAM, "helper(*", 0, 3)),
        "helper :: (t: *Thing) -> int"
    );
    assert!(
        hover(&s, after(PROGRAM, "thing: Thing", 0, 1))
            .starts_with("Thing :: struct { alpha: s64; beta: float32; label: string; }"),
    );
    assert_eq!(hover(&s, after(PROGRAM, "t.alpha", 0, 1)), "alpha: s64");
    // Bodies nothing calls are checked too.
    assert_eq!(
        hover(&s, after(PROGRAM, "return doubled", 0, 1)),
        "doubled: s64"
    );
    assert!(hover(&s, after(PROGRAM, "print(", 0, 2)).starts_with("print :: "));
}

#[test]
fn completion_offers_locals_globals_and_imports_while_typing() {
    let mut s = session();
    // A half-typed statement: the text does not parse.
    let text = PROGRAM.replace("    print(\"%\\n\", total);\n", "    co\n");
    s.open(uri(), 1, text.clone()).unwrap();
    let items = labels(&s, after(&text, "    co\n", 0, 1));
    assert!(
        items
            .iter()
            .any(|(l, k, d)| l == "counter" && *k == CompletionKind::Variable && d == "s64"),
        "{items:?}"
    );
    let text = PROGRAM.replace("    print(\"%\\n\", total);\n", "    \n");
    s.change(
        &uri(),
        2,
        &[TextChange {
            range: None,
            range_length: None,
            text: text.clone(),
        }],
    )
    .unwrap();
    let items = labels(
        &s,
        after(&text, "total := counter + helper(*thing);\n    ", 0, 0),
    );
    let has =
        |name: &str, kind: CompletionKind| items.iter().any(|(l, k, _)| l == name && *k == kind);
    assert!(has("total", CompletionKind::Variable));
    assert!(has("thing", CompletionKind::Variable));
    assert!(has("helper", CompletionKind::Function));
    assert!(has("Thing", CompletionKind::Struct));
    assert!(has("print", CompletionKind::Function), "Basic's exports");
    assert!(has("for", CompletionKind::Keyword));
    // Locals of other procedures are not visible.
    assert!(!items.iter().any(|(l, ..)| l == "doubled"));
}

#[test]
fn completion_after_a_dot_lists_members() {
    let mut s = session();
    let text = PROGRAM.replace("    print(\"%\\n\", total);\n", "    thing.\n");
    s.open(uri(), 1, text.clone()).unwrap();
    let items = labels(&s, after(&text, "    thing.", 0, 0));
    let names: Vec<&str> = items.iter().map(|(l, ..)| l.as_str()).collect();
    assert_eq!(names, ["alpha", "beta", "label"], "{items:?}");
    let text = PROGRAM.replace("    print(\"%\\n\", total);\n", "    thing.label.co\n");
    s.change(
        &uri(),
        2,
        &[TextChange {
            range: None,
            range_length: None,
            text: text.clone(),
        }],
    )
    .unwrap();
    let items = labels(&s, after(&text, "thing.label.co", 0, 0));
    assert_eq!(
        items.iter().map(|(l, ..)| l.as_str()).collect::<Vec<_>>(),
        ["count"]
    );
    let text = PROGRAM.replace("    print(\"%\\n\", total);\n", "    m := Mode.\n");
    s.change(
        &uri(),
        3,
        &[TextChange {
            range: None,
            range_length: None,
            text: text.clone(),
        }],
    )
    .unwrap();
    let items = labels(&s, after(&text, "Mode.", 0, 0));
    let names: Vec<&str> = items.iter().map(|(l, ..)| l.as_str()).collect();
    assert!(
        names.contains(&"IDLE") && names.contains(&"RUNNING"),
        "{items:?}"
    );
}

#[test]
fn hover_works_while_another_line_is_half_typed() {
    let mut s = session();
    let text = PROGRAM.replace(
        "    print(\"%\\n\", total);\n",
        "    print(\"%\\n\", total);\n    thing.\n",
    );
    s.open(uri(), 1, text.clone()).unwrap();
    assert_eq!(hover(&s, after(&text, "%\\n\", total", 0, 1)), "total: s64");
}

#[test]
fn completion_after_a_hash_lists_directives() {
    let mut s = session();
    let text = PROGRAM.replace("    print(\"%\\n\", total);\n", "    #ru\n");
    s.open(uri(), 1, text.clone()).unwrap();
    let items = labels(&s, after(&text, "#ru", 0, 0));
    assert_eq!(
        items.iter().map(|(l, ..)| l.as_str()).collect::<Vec<_>>(),
        ["#run", "#runtime_support"]
    );
    let text = PROGRAM.replace("    print(\"%\\n\", total);\n", "    #\n");
    s.change(
        &uri(),
        2,
        &[TextChange {
            range: None,
            range_length: None,
            text: text.clone(),
        }],
    )
    .unwrap();
    let items = labels(&s, after(&text, "    #", 0, 0));
    assert!(
        items
            .iter()
            .any(|(l, k, _)| l == "#import" && *k == CompletionKind::Keyword)
    );
    assert!(items.iter().any(|(l, ..)| l == "#insert"));
}

#[test]
fn definition_reaches_loaded_files_and_the_stdlib() {
    let mut s = session();
    let other = DocumentUri::parse("file:///lsp-semantic-test/other.jai").unwrap();
    let main = concat!(
        "#import \"Basic\";\n",
        "#load \"other.jai\";\n",
        "main :: () {\n",
        "    x := twice(3);\n",
        "    print(\"%\\n\", x);\n",
        "}\n",
    );
    s.open(
        other.clone(),
        1,
        "twice :: (n: int) -> int { return n * 2; }\n".into(),
    )
    .unwrap();
    s.open(uri(), 1, main.into()).unwrap();
    let local = s.definition(&uri(), after(main, "twice", 0, 2)).unwrap();
    assert_eq!(local.len(), 1);
    assert_eq!(local[0].uri, other.as_str());
    assert_eq!(
        (local[0].range.start.character, local[0].range.end.character),
        (0, 5)
    );
    let x = s.definition(&uri(), after(main, ", x", 0, 0)).unwrap();
    assert_eq!(x[0].uri, uri().as_str());
    assert_eq!(x[0].range.start.line, 3);
    let print = s.definition(&uri(), after(main, "print", 0, 1)).unwrap();
    assert!(!print.is_empty());
    assert!(
        print.iter().all(|l| l.uri.contains("/stdlib/")),
        "{print:?}"
    );
    // Every overload (and alias) lands on its declared name.
    for at in &print {
        let source = s.source(&DocumentUri::parse(&at.uri).unwrap()).unwrap();
        let line = source.lines().nth(at.range.start.line as usize).unwrap();
        assert!(
            line[at.range.start.character as usize..].starts_with("print"),
            "{line}"
        );
        assert!(at.range.end.character > at.range.start.character, "{line}");
    }
}

#[test]
fn completion_inside_load_and_import_strings_lists_paths_and_modules() {
    let mut s = session();
    let helper = DocumentUri::parse("file:///lsp-semantic-test/util/helpers.jai").unwrap();
    let local = DocumentUri::parse("file:///lsp-semantic-test/other.jai").unwrap();
    s.open(helper, 1, "x :: 1;\n".into()).unwrap();
    s.open(local, 1, "y :: 2;\n".into()).unwrap();
    let text = "#import \"Bas\";\n#load \"\";\n#load \"util/h\";\n";
    s.open(uri(), 1, text.into()).unwrap();
    let names = |at| -> Vec<String> { labels(&s, at).into_iter().map(|l| l.0).collect() };
    assert_eq!(names(after(text, "\"Bas", 0, 0)), ["Base64", "Basic"]);
    assert_eq!(names(after(text, "#load \"", 0, 0)), ["other.jai", "util/"]);
    assert_eq!(names(after(text, "util/h", 0, 0)), ["helpers.jai"]);
}

/// The type checker's error is published, also for a procedure nothing calls (the program's
/// own code is checked whether or not it is used), at the offending code; fixed, it goes away.
// rules: dce.2
#[test]
fn type_error_in_an_unused_procedure_is_published() {
    let text = "#import \"Basic\";\nunused :: () {\n    x: int = \"text\";\n}\nmain :: () {}\n";
    let checks = |s: &Session| -> Vec<jai_language_server::Diagnostic> {
        s.diagnostics(&uri())
            .unwrap()
            .into_iter()
            .filter(|d| d.code == jai_language_server::DiagnosticCode::Check)
            .collect()
    };
    let mut s = session();
    s.open(uri(), 1, text.into()).unwrap();
    let found = checks(&s);
    assert_eq!(found.len(), 1, "{found:?}");
    assert!(found[0].message.contains("type mismatch"), "{found:?}");
    assert_eq!(
        found[0].range.start,
        // The value that does not fit, not the start of the declaration.
        Position {
            line: 2,
            character: 13
        }
    );
    let fixed = TextChange {
        range: None,
        range_length: None,
        text: text.replace("\"text\"", "3"),
    };
    s.change(&uri(), 2, &[fixed]).unwrap();
    assert!(checks(&s).is_empty());
}
