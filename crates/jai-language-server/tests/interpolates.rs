//! `@Interpolates` holes are code: hover, definition, references, rename, completion, semantic
//! tokens and diagnostics reach the expressions inside the string.
use jai_language_server::{DiagnosticSeverity, Position, SemanticTokenKind};
use test_support::lsp::{at_offset, cursor, open, position, session, uri};

/// Line 6 holds the call; `%user_id` (twice: once in code, once inside quotes) and `%user.name`.
const SOURCE: &str = "#import \"Basic\";\n\
    run_query :: (db: int, sql: string, args: ..Any) -> int { return args.count; } @Interpolates(\"sql\")\n\
    User :: struct { id: int; name: string; }\n\
    main :: () {\n\
    \x20   user_id := 4;\n\
    \x20   user: User;\n\
    \x20   n := run_query(1, \"select * from users where id = %user_id and n = '%user_id' and name = %user.name\");\n\
    \x20   print(\"%\\n\", n);\n\
    }\n";

fn hole(nth: usize, offset: usize) -> Position {
    // `nth` counts `%user` spellings on the call line.
    let line = SOURCE.lines().nth(6).unwrap();
    let base = SOURCE.find(line).unwrap();
    let rel = line.match_indices("%user").nth(nth).unwrap().0 + offset;
    position(SOURCE, base + rel)
}

fn text_at(range: jai_language_server::Range) -> (u32, u32, u32) {
    (range.start.line, range.start.character, range.end.character)
}

#[test]
fn hover_on_a_hole_names_the_variable() {
    let s = open(SOURCE);
    let hover = s.hover(&uri(), hole(0, 3)).unwrap().expect("a hover");
    assert!(hover.contents.value.contains("user_id"), "{hover:?}");
    assert!(hover.contents.value.contains("s64") || hover.contents.value.contains("int"));
    // The hover covers the name in the string, not the string.
    let line = SOURCE.lines().nth(6).unwrap();
    let name = line.find("%user_id").unwrap() + 1;
    assert_eq!(text_at(hover.range), (6, name as u32, name as u32 + 7));
}

#[test]
fn go_to_definition_from_a_hole() {
    let s = open(SOURCE);
    let found = s.definition(&uri(), hole(0, 2)).unwrap();
    assert_eq!(found.len(), 1);
    assert_eq!(found[0].range.start.line, 4);
    // A field inside the string finds its declaration too.
    let found = s
        .definition(&uri(), at_offset(SOURCE, "%user.name", 7))
        .unwrap();
    assert_eq!(found[0].range.start.line, 2);
}

#[test]
fn references_and_rename_edit_the_string() {
    let mut s = session();
    s.open(uri(), 1, SOURCE.into()).unwrap();
    let refs = s.references(&uri(), hole(0, 2), true).unwrap();
    // The declaration and the one hole; the `'%user_id'` inside quotes is text.
    let lines: Vec<u32> = refs.iter().map(|l| l.range.start.line).collect();
    assert_eq!(lines, [4, 6], "{refs:?}");
    let range = s
        .prepare_rename(&uri(), hole(0, 2))
        .unwrap()
        .expect("renamable");
    assert_eq!(range.end.character - range.start.character, 7);
    let edits = s.rename(&uri(), hole(0, 2), "uid").unwrap().expect("edits");
    assert_eq!(edits[0].1.len(), 2);
    let mut text = SOURCE.to_string();
    let mut starts: Vec<_> = edits[0].1.iter().collect();
    starts.sort_by_key(|e| std::cmp::Reverse((e.range.start.line, e.range.start.character)));
    for edit in starts {
        let line_start: usize = SOURCE
            .split_inclusive('\n')
            .take(edit.range.start.line as usize)
            .map(str::len)
            .sum();
        let a = line_start + edit.range.start.character as usize;
        let b = line_start + edit.range.end.character as usize;
        text.replace_range(a..b, &edit.new_text);
    }
    assert!(text.contains("id = %uid and"), "{text}");
    assert!(text.contains("n = '%user_id'"), "{text}");
    assert!(text.contains("uid := 4;"), "{text}");
}

#[test]
fn completion_after_a_percent_offers_the_callers_names() {
    let (text, at) = cursor(&SOURCE.replace("%user_id and", "%|user_id and"));
    let s = open(&text);
    let list = s.completion(&uri(), at).unwrap();
    let labels: Vec<&str> = list.items.iter().map(|i| i.label.as_str()).collect();
    assert!(labels.contains(&"user_id"), "{labels:?}");
    assert!(labels.contains(&"user"), "{labels:?}");
    assert!(!labels.contains(&"run_query_missing"));
}

#[test]
fn completion_of_a_typed_prefix_and_members() {
    let typed = "#import \"Basic\";\n\
        run_query :: (db: int, sql: string, args: ..Any) -> int { return args.count; } @Interpolates(\"sql\")\n\
        User :: struct { id: int; name: string; }\n\
        main :: () {\n\
        \x20   user_id := 4;\n\
        \x20   user: User;\n\
        \x20   n := run_query(1, \"select %user_id, %user.|\");\n\
        }\n";
    let (text, at) = cursor(typed);
    let s = open(&text);
    let list = s.completion(&uri(), at).unwrap();
    let labels: Vec<&str> = list.items.iter().map(|i| i.label.as_str()).collect();
    assert!(
        labels.contains(&"name") && labels.contains(&"id"),
        "{labels:?}"
    );
    assert!(!labels.contains(&"user_id"), "{labels:?}");
    // A prefix filters.
    let (text, at) = cursor(&typed.replace("%user.|", "%us|"));
    let s = open(&text);
    let labels: Vec<String> = s
        .completion(&uri(), at)
        .unwrap()
        .items
        .into_iter()
        .map(|i| i.label)
        .collect();
    assert!(
        labels.contains(&"user_id".to_string()) && labels.contains(&"user".to_string()),
        "{labels:?}"
    );
    assert!(!labels.contains(&"main".to_string()), "{labels:?}");
}

#[test]
fn quoted_sql_is_not_code() {
    let s = open(SOURCE);
    assert!(s.in_interpolation_hole(&uri(), hole(0, 3)));
    let line = SOURCE.lines().nth(6).unwrap();
    let base = SOURCE.find(line).unwrap();
    let quoted = line.find("'%user_id'").unwrap() + 4;
    assert!(!s.in_interpolation_hole(&uri(), position(SOURCE, base + quoted)));
    assert!(
        s.hover(&uri(), position(SOURCE, base + quoted))
            .unwrap()
            .is_none()
    );
}

#[test]
fn holes_are_highlighted_as_code() {
    let s = open(SOURCE);
    let tokens = s.semantic_tokens(&uri()).unwrap();
    let on_line: Vec<_> = tokens.iter().filter(|t| t.position.line == 6).collect();
    let line = SOURCE.lines().nth(6).unwrap();
    let col = |needle: &str, nth: usize| line.match_indices(needle).nth(nth).unwrap().0 as u32;
    let has = |c: u32, len: u32, kind: SemanticTokenKind| {
        on_line
            .iter()
            .any(|t| t.position.character == c && t.length == len && t.kind == kind)
    };
    // `%` is an operator, `user_id` a variable.
    assert!(
        has(col("%user_id", 0), 1, SemanticTokenKind::Operator),
        "{on_line:?}"
    );
    assert!(
        has(col("%user_id", 0) + 1, 7, SemanticTokenKind::Variable),
        "{on_line:?}"
    );
    // The field after the dot is a property.
    assert!(
        has(col("%user.name", 0) + 6, 4, SemanticTokenKind::Property),
        "{on_line:?}"
    );
    // The quoted one stays string text: no token starts at its name.
    assert!(
        !on_line
            .iter()
            .any(|t| t.position.character == col("'%user_id'", 0) + 2),
        "{on_line:?}"
    );
}

#[test]
fn errors_in_a_hole_are_reported_there() {
    let source = SOURCE.replace("%user.name", "%missing.name");
    let s = open(&source);
    let diagnostics = s.diagnostics(&uri()).unwrap();
    let errors: Vec<_> = diagnostics
        .iter()
        .filter(|d| d.severity == DiagnosticSeverity::Error)
        .collect();
    let line = source.lines().nth(6).unwrap();
    let col = line.find("missing").unwrap() as u32;
    assert!(
        errors
            .iter()
            .any(|d| d.range.start.line == 6 && d.range.start.character == col),
        "{errors:?}"
    );
}
