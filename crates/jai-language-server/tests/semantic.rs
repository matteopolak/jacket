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
