//! A declaration quoted without type information is cut by what it writes, not by its layout.
use jai_language_server::{DocumentUri, Limits, Position, Session};

fn quoted(text: &str) -> String {
    let uri = DocumentUri::parse("file:///lsp-test/main.jai").unwrap();
    let mut s = Session::new(Limits::default());
    s.open(uri.clone(), 1, text.into()).unwrap();
    let at = Position {
        line: 0,
        character: 1,
    };
    s.hover(&uri, at).unwrap().expect("hover").contents.value
}

fn without_space(text: &str) -> String {
    text.split_whitespace().collect()
}

#[test]
fn blank_lines_do_not_move_where_a_long_declaration_is_cut() {
    let lines: String = (0..60).map(|i| format!("    value{i} := {i};\n")).collect();
    let tight = format!("big :: (heap: *$T) {{\n{lines}}}\n");
    let spaced = tight.replace(";\n", ";\n\n\n");
    let (tight, spaced) = (quoted(&tight), quoted(&spaced));
    assert!(tight.contains("Source syntax declaration"), "{tight}");
    assert!(!tight.contains("value59"), "cut: {tight}");
    assert_eq!(without_space(&tight), without_space(&spaced));
}
