//! What a call returns or stores, as the lifetime rules see it through wrappers.
use super::*;

/// The procedures (top-level declarations) holding a finding, in source order.
fn finding_procedures(lints: &[Lint], text: &str) -> Vec<String> {
    lints
        .iter()
        .map(|l| {
            text[..l.start]
                .lines()
                .rev()
                .find(|line| !line.starts_with(' ') && line.contains(" :: "))
                .and_then(|line| line.split(" :: ").next())
                .unwrap_or("")
                .to_string()
        })
        .collect()
}

#[test]
fn lifetime_wrappers_return_aliases_or_fresh_storage() {
    let case = repo().join("tests/lint/pointer_lifetime/resources");
    for (file, expected) in [
        (
            "return-aliases.jai",
            vec![
                "alias_freed_source",
                "alias_freed_through_two",
                "alias_of_field",
                "second_field_alias",
                "picks_first_freed",
            ],
        ),
        ("captured-read-returns.jai", vec!["pure_projection"]),
    ] {
        let path = case.join(file);
        let (lints, text) = lint(
            &path,
            &[
                ("use_after_free", Level::Warn),
                ("double_free", Level::Warn),
            ],
        );
        assert_eq!(
            finding_procedures(&lints, &text),
            expected,
            "{file}\n{}",
            rendered(&lints, &text, &path)
        );
    }
}
