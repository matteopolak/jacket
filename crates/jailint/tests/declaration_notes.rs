//! `unused_note` leaves the declaration notes the rules read alone: its list
//! (`jailint::LINT_NOTES`) is the tables of `docs/tools/jailint.md`.
use std::path::Path;

#[test]
fn unused_note_knows_every_declaration_note() {
    let doc = std::fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docs/tools/jailint.md"),
    )
    .expect("docs/tools/jailint.md is readable");
    let mut documented: Vec<&str> = doc
        .lines()
        .filter_map(|l| l.strip_prefix("| `@"))
        .flat_map(|l| l.split(" | ").next().unwrap_or_default().split('@'))
        .map(|n| {
            n.split(|c: char| !c.is_ascii_alphanumeric() && c != '_')
                .next()
                .unwrap_or_default()
        })
        .filter(|n| !n.is_empty())
        .collect();
    documented.sort_unstable();
    documented.dedup();
    let mut listed = jailint::LINT_NOTES.to_vec();
    listed.sort_unstable();
    assert_eq!(listed, documented);
}
