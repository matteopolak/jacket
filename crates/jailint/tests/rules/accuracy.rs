use super::*;

#[test]
fn lifetime_local_fixed_views_only_escape_when_their_holder_does() {
    let path = repo().join("tests/lint/pointer_lifetime/strict/local-fixed-views.jai");
    let (lints, text) = lint(&path, &[("returned_local_address", Level::Warn)]);
    assert_eq!(lints.len(), 1, "{}", rendered(&lints, &text, &path));
    assert!(lints[0].start > text.find("bad_view").unwrap());
}

#[test]
fn lifetime_temporary_release_help_describes_the_reset() {
    let path = repo().join("tests/lint/pointer_lifetime/strict/reset-help.jai");
    let (lints, text) = lint(&path, &[("use_after_free", Level::Warn)]);
    assert_eq!(lints.len(), 1, "{}", rendered(&lints, &text, &path));
    assert!(
        lints[0]
            .help
            .as_deref()
            .unwrap()
            .contains("use it before the reset")
    );
}

#[test]
fn lifetime_conditional_growth_wrappers_have_a_strict_witness() {
    let path = repo().join("tests/lint/pointer_lifetime/strict/conditional-growth.jai");
    let levels = [("pointer_lifetime", Level::Warn)];
    let (must, text) = lint(&path, &levels);
    assert!(must.is_empty(), "{}", rendered(&must, &text, &path));
    let (strict, text) = lint_with(
        &path,
        &levels,
        Config {
            lifetime_mode: Some(jacket_config::LifetimeMode::Strict),
            ..Config::default()
        },
    );
    assert_eq!(strict.len(), 1, "{}", rendered(&strict, &text, &path));
}

#[test]
fn lifetime_destroy_labels_the_original_allocation() {
    let path = repo().join("tests/lint/pointer_lifetime/strict/destroy-origin.jai");
    let (lints, text) = lint(&path, &[("use_after_free", Level::Warn)]);
    assert_eq!(lints.len(), 1, "{}", rendered(&lints, &text, &path));
    let origin = lints[0]
        .secondary
        .iter()
        .find(|s| s.message == "storage derived here")
        .unwrap();
    assert_eq!(&text[origin.start..origin.end], "New(Box)");
}

#[test]
fn lifetime_counted_loops_exclude_zero_iteration_bypasses() {
    let path = repo().join("tests/lint/pointer_lifetime/strict/counted-growth.jai");
    for mode in [
        jacket_config::LifetimeMode::Must,
        jacket_config::LifetimeMode::Strict,
    ] {
        let (lints, text) = lint_with(
            &path,
            &[("pointer_lifetime", Level::Warn)],
            Config {
                lifetime_mode: Some(mode),
                ..Config::default()
            },
        );
        assert_eq!(
            lints.len(),
            if mode == jacket_config::LifetimeMode::Must {
                1
            } else {
                4
            },
            "{}",
            rendered(&lints, &text, &path)
        );
        assert!(lints[0].start < text.find("empty_range").unwrap());
        assert!(
            lints
                .iter()
                .all(|l| l.start < text.find("empty_range").unwrap()
                    || l.start > text.find("few_adds").unwrap())
        );
    }
}
