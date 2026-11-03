//! Return-origin and Windows regressions for the shared lifetime engine.
use super::*;

#[test]
fn lifetime_return_origins_distinguish_heap_copies_and_temporary_results() {
    let path = repo().join("tests/lint/pointer_lifetime/strict/return-origins.jai");
    for mode in [
        jacket_config::LifetimeMode::Must,
        jacket_config::LifetimeMode::Strict,
    ] {
        let (findings, text) = lint_with(
            &path,
            &[("temp_storage_escape", Level::Warn)],
            Config {
                lifetime_mode: Some(mode),
                ..Config::default()
            },
        );
        assert_eq!(findings.len(), 6, "{}", rendered(&findings, &text, &path));
        assert!(
            findings
                .iter()
                .all(|f| f.start > text.find("bad_temporary ::").unwrap()),
            "{}",
            rendered(&findings, &text, &path)
        );
    }
}


#[test]
fn lifetime_windows_stdlib_shapes_preserve_headers_and_copied_results() {
    let path = repo().join("tests/lint/pointer_lifetime/strict/windows-stdlib.jai");
    let mut target = jaic::sema::Options::host();
    target.os = jaic::target::TargetOs::Windows;
    target.cpu = jaic::target::TargetCpu::X64;
    let rules = [
        ("use_after_free", Level::Warn),
        ("returned_local_address", Level::Warn),
        ("temp_storage_escape", Level::Warn),
        ("large_stack_array", Level::Warn),
    ];
    let (findings, text) = lint_target(&path, &rules, Config::default(), target);
    assert!(findings.is_empty(), "{}", rendered(&findings, &text, &path));
}

#[test]
fn lifetime_mixed_return_contracts_remain_possible_without_tainting_heap_copies() {
    let path = repo().join("tests/lint/pointer_lifetime/strict/return-defaults.jai");
    for (mode, expected) in [
        (jacket_config::LifetimeMode::Must, 1),
        (jacket_config::LifetimeMode::Strict, 3),
    ] {
        let (findings, text) = lint_with(
            &path,
            &[("temp_storage_escape", Level::Warn)],
            Config {
                lifetime_mode: Some(mode),
                ..Config::default()
            },
        );
        assert_eq!(
            findings.len(),
            expected,
            "{}",
            rendered(&findings, &text, &path)
        );
        assert!(
            findings
                .iter()
                .all(|f| f.start > text.find("retain_nullable ::").unwrap()),
            "{}",
            rendered(&findings, &text, &path)
        );
    }
}

#[test]
fn lifetime_failed_windows_close_is_not_a_must_release() {
    let path = repo().join("tests/lint/pointer_lifetime/strict/failed-close.jai");
    let mut target = jaic::sema::Options::host();
    target.os = jaic::target::TargetOs::Windows;
    target.cpu = jaic::target::TargetCpu::X64;
    let (findings, text) = lint_target(
        &path,
        &[
            ("double_free", Level::Warn),
            ("use_after_free", Level::Warn),
        ],
        Config::default(),
        target,
    );
    assert!(findings.is_empty(), "{}", rendered(&findings, &text, &path));
}

#[test]
fn lifetime_reassigned_heap_header_does_not_alias_its_contained_storage() {
    let path = repo().join("tests/lint/pointer_lifetime/strict/reassigned-header.jai");
    for mode in [
        jacket_config::LifetimeMode::Must,
        jacket_config::LifetimeMode::Strict,
    ] {
        let (findings, text) = lint_with(
            &path,
            &[
                ("double_free", Level::Warn),
                ("use_after_free", Level::Warn),
            ],
            Config {
                lifetime_mode: Some(mode),
                ..Config::default()
            },
        );
        assert!(findings.is_empty(), "{}", rendered(&findings, &text, &path));
    }
}

#[test]
fn lifetime_returned_address_aliases_preserve_their_original_generation() {
    let path = repo().join("tests/lint/pointer_lifetime/strict/aligned-alias.jai");
    for mode in [
        jacket_config::LifetimeMode::Must,
        jacket_config::LifetimeMode::Strict,
    ] {
        let (findings, text) = lint_with(
            &path,
            &[("use_after_free", Level::Warn)],
            Config {
                lifetime_mode: Some(mode),
                ..Config::default()
            },
        );
        assert_eq!(findings.len(), 3, "{}", rendered(&findings, &text, &path));
        assert!(
            findings
                .iter()
                .all(|f| ["alias.*", "forwarded.*", "old_temporary.*"]
                    .iter()
                    .any(|prefix| text[f.start..].starts_with(prefix))),
            "{}",
            rendered(&findings, &text, &path)
        );
    }
}
