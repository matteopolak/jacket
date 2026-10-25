//! A user-written polymorphic struct whose body `#insert` builds from a long string argument:
//! hover writes a value's struct type out as its fields under their `///` docs, a field's own
//! hover reads its docs from the inserted text, and diagnostics cut the argument short.
use jai_language_server::DiagnosticCode;
mod common;
use common::{at, open, uri};

const PROGRAM: &str = "Row :: struct ($shape: string) { #insert shape; }

main :: () {
    row: Row(\"/// users.id: bigint, not null\\nid: s64;\\n/// users.email: text, not null\\nemail: string;\\n\");
    wrong: int = row;
    print_id(row.id);
    spot: Spot;
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
