#[path = "../examples/support/edit_coverage.rs"]
mod edit_coverage;

#[test]
fn every_line_suffix_in_original_fixtures_preserves_independent_features() {
    edit_coverage::set_progress(false);
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../tests/lsp/partial");
    for name in [
        "standalone.jai",
        "custom_iteration.jai",
        "opaque.jai",
        "lists.jai",
    ] {
        let file = root.join(name);
        let source = std::fs::read_to_string(&file).unwrap();
        let report = edit_coverage::check(&file, &source, None, 0);
        assert!(report.edits > 40, "{report:?}");
        assert!(report.failures.is_empty(), "{report:#?}");
    }
}

#[test]
fn failed_pending_generator_keeps_real_errors_and_independent_features() {
    edit_coverage::set_progress(false);
    let file = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/lsp/partial/pending.jai");
    let source = std::fs::read_to_string(&file).unwrap();
    let report = edit_coverage::check(&file, &source, None, 0);
    assert!(report.baseline_semantic_errors > 0, "{report:?}");
    assert!(report.edits >= 20, "{report:?}");
    assert!(report.failures.is_empty(), "{report:#?}");
}
