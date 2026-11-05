//! Stable public heading identifiers, shared by Markdown ingestion and export fixtures.
/// GitHub-style heading base: lowercase letters, digits, underscores and hyphens.
/// For example, `API: Point.x` becomes `api-pointx`.
pub(super) fn slug(text: &str) -> String {
    text.trim()
        .to_lowercase()
        .chars()
        .filter_map(|c| match c {
            c if c.is_alphanumeric() || c == '_' || c == '-' => Some(c),
            ' ' => Some('-'),
            _ => None,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    #[test]
    fn anchors_follow_githubs_rule() {
        assert_eq!(super::slug("Per-test options (v2)"), "per-test-options-v2");
    }
}
