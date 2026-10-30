//! A user-written polymorphic struct whose body `#insert` builds from a long string argument:
//! hover writes a value's struct type out as its fields under their `///` docs, a field's own
//! hover reads its docs from the inserted text, diagnostics cut the argument short and an inlay
//! hint names the polymorphic struct alone.
use jai_language_server::DiagnosticCode;
use jai_language_server::Range;
use test_support::lsp::{at, open, position, uri};

const PROGRAM: &str = "Row :: struct ($shape: string) { #insert shape; }

main :: () {
    row: Row(\"/// users.id: bigint, not null\\nid: s64;\\n/// users.email: text, not null\\nemail: string;\\n\");
    wrong: int = row;
    print_id(row.id);
    spot: Spot;
    copy := row;
}

print_id :: (id: s64) {}

Spot :: struct {
    /// Across.
    x: float;
    y: float;
}
";

fn hover(marker: &str) -> String {
    let s = open(PROGRAM);
    s.hover(&uri(), at(PROGRAM, marker))
        .unwrap()
        .expect("hover")
        .contents
        .value
}

#[test]
fn hover_shows_the_fields_under_their_docs() {
    let text = hover("row: Row");
    let fields = "row: Row {
    /// users.id: bigint, not null
    id: s64;
    /// users.email: text, not null
    email: string;
}";
    assert!(text.contains(fields), "{text}");
    assert!(!text.contains("Row(\""), "{text}");
}

#[test]
fn a_field_use_shows_the_docs_written_in_the_inserted_text() {
    let text = hover("id);");
    assert!(text.contains("id: s64"), "{text}");
    assert!(text.contains("users.id: bigint, not null"), "{text}");
}

#[test]
fn an_ordinary_struct_value_lists_its_fields() {
    let text = hover("spot: Spot");
    assert!(
        text.contains("spot: Spot {\n    /// Across.\n    x: float32;\n    y: float32;\n}"),
        "{text}"
    );
}

#[test]
fn diagnostics_cut_the_argument_short() {
    let s = open(PROGRAM);
    let messages: Vec<String> = s
        .diagnostics(&uri())
        .unwrap()
        .into_iter()
        .filter(|d| d.code == DiagnosticCode::Check)
        .map(|d| d.message)
        .collect();
    assert!(
        messages
            .iter()
            .any(|m| m.contains("Row(\"/// users.id: bigint, no…\")")),
        "{messages:?}"
    );
    assert!(
        messages.iter().all(|m| !m.contains("email")),
        "{messages:?}"
    );
}

#[test]
fn an_inlay_hint_names_the_polymorphic_struct_alone() {
    let s = open(PROGRAM);
    let all = Range {
        start: position(PROGRAM, 0),
        end: position(PROGRAM, PROGRAM.len()),
    };
    let hints = s.inlay_hints(&uri(), all).unwrap();
    let copy = at(PROGRAM, "copy :=");
    let label = hints
        .iter()
        .find(|h| h.position.line == copy.line)
        .map(|h| h.label.clone());
    assert_eq!(label.as_deref(), Some(": Row"), "{hints:?}");
}
