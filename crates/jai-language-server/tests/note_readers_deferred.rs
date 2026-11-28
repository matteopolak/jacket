//! A note that only a generic instance made from a library body the editor check leaves
//! unchecked reads still leads to that reader: the body is checked for the request.
use test_support::lsp::{open_file, position};

#[test]
fn a_note_read_by_a_generic_instance_in_a_skipped_body_leads_to_its_reader() {
    let folder = test_support::repo::root().join("examples/http");
    let (s, uri, text) = open_file(&folder, "notes.jai");
    let at = text.find("@json(default)").unwrap() + 1;
    let found = s.definition(&uri, position(&text, at)).unwrap();

    assert!(
        found
            .iter()
            .any(|l| l.uri.as_str().contains("Extensions/Json")),
        "{found:?}"
    );
}
