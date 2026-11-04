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

#[test]
fn lifetime_growth_chains_label_calls_instead_of_element_stores() {
    let path = repo().join("tests/lint/pointer_lifetime/strict/growth-chain.jai");
    let (lints, text) = lint(&path, &[("pointer_lifetime", Level::Warn)]);
    assert_eq!(lints.len(), 1, "{}", rendered(&lints, &text, &path));
    let labels: Vec<_> = lints[0]
        .secondary
        .iter()
        .filter(|s| s.message == "invalidation inferred through this call")
        .collect();
    assert!(!labels.is_empty(), "{}", rendered(&lints, &text, &path));
    assert!(
        labels
            .iter()
            .all(|s| !s.text[s.start..s.end].contains(" = ")),
        "{}",
        rendered(&lints, &text, &path)
    );
}

#[test]
fn lifetime_possible_temporary_results_are_checked_at_the_caller() {
    let path = repo().join("tests/lint/pointer_lifetime/strict/temporary-result-summary.jai");
    let levels = [("temp_storage_escape", Level::Warn)];
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
    assert!(strict[0].start > text.find("retain_result").unwrap());
}

#[test]
fn lifetime_repeated_branches_do_not_repeat_identical_witness_labels() {
    let path = repo().join("tests/lint/pointer_lifetime/strict/counted-growth.jai");
    let (lints, text) = lint_with(
        &path,
        &[("pointer_lifetime", Level::Warn)],
        Config {
            lifetime_mode: Some(jacket_config::LifetimeMode::Strict),
            ..Config::default()
        },
    );
    assert_eq!(lints.len(), 4, "{}", rendered(&lints, &text, &path));
    for lint in lints {
        let mut seen = jaic::fxhash::HashSet::default();
        for label in lint.secondary {
            assert!(
                seen.insert((label.path, label.start, label.end, label.message)),
                "duplicate witness label"
            );
        }
    }
}

#[test]
fn lifetime_entrypoint_storage_lasts_until_process_exit() {
    let path = repo().join("tests/lint/pointer_lifetime/strict/entrypoint-retention.jai");
    let (lints, text) = lint(&path, &[("returned_local_address", Level::Warn)]);
    assert_eq!(lints.len(), 2, "{}", rendered(&lints, &text, &path));
    assert!(!lints.iter().any(|l| text[l.start..l.end].contains("outer")));
}

#[test]
fn lifetime_compile_time_results_are_frozen_before_the_run_ends() {
    let path = repo().join("tests/lint/pointer_lifetime/strict/frozen-retention.jai");
    let (lints, text) = lint(&path, &[("returned_local_address", Level::Warn)]);
    assert!(lints.is_empty(), "{}", rendered(&lints, &text, &path));
}

#[test]
fn lifetime_runtime_calls_keep_ordinary_stack_lifetimes() {
    for name in [
        "mixed-run-retention",
        "frozen-callback-retention",
        "callable-entrypoint-retention",
    ] {
        let path = repo().join(format!("tests/lint/pointer_lifetime/strict/{name}.jai"));
        let (lints, text) = lint(&path, &[("returned_local_address", Level::Warn)]);
        assert_eq!(lints.len(), 1, "{}", rendered(&lints, &text, &path));
    }
}

#[test]
fn lifetime_frozen_results_still_reject_explicit_temporary_resets() {
    let path = repo().join("tests/lint/pointer_lifetime/strict/frozen-reset-retention.jai");
    let (lints, text) = lint(&path, &[("use_after_free", Level::Warn)]);
    assert_eq!(lints.len(), 2, "{}", rendered(&lints, &text, &path));
}

#[test]
fn lifetime_entrypoint_temporaries_only_need_to_survive_until_process_exit() {
    let path = repo().join("tests/lint/pointer_lifetime/strict/entrypoint-temporary-retention.jai");
    let (lints, text) = lint(&path, &[("temp_storage_escape", Level::Warn)]);
    assert_eq!(lints.len(), 1, "{}", rendered(&lints, &text, &path));
    assert!(lints[0].start > text.find("retain ::").unwrap());
}

#[test]
fn lifetime_custom_runtimes_do_not_get_process_entry_lifetimes() {
    let path = repo().join("tests/lint/pointer_lifetime/strict/entrypoint-retention.jai");
    let mut options = jaic::sema::Options::host();
    options.runtime_entry_point = false;
    let (lints, text) = lint_target(
        &path,
        &[("returned_local_address", Level::Warn)],
        Config::default(),
        options,
    );
    assert_eq!(lints.len(), 3, "{}", rendered(&lints, &text, &path));
}

#[test]
fn lifetime_pointer_sinks_resolve_local_owners_without_trusting_rebinding() {
    let path = repo().join("tests/lint/pointer_lifetime/strict/local-pointer-sink.jai");
    let (lints, text) = lint(&path, &[("returned_local_address", Level::Warn)]);
    assert_eq!(lints.len(), 3, "{}", rendered(&lints, &text, &path));
    assert!(
        lints
            .iter()
            .all(|l| l.start > text.find("rebound_sink").unwrap()),
        "{}",
        rendered(&lints, &text, &path)
    );
}

#[test]
fn lifetime_expanded_storage_belongs_to_the_callers_block() {
    let path = repo().join("tests/lint/pointer_lifetime/strict/expanded-storage.jai");
    let (lints, text) = lint(&path, &[("returned_local_address", Level::Warn)]);
    assert_eq!(lints.len(), 2, "{}", rendered(&lints, &text, &path));
    assert!(
        lints
            .iter()
            .all(|l| l.start > text.find("bad_return").unwrap())
    );
}

#[test]
fn lifetime_pointer_field_results_keep_their_temporary_origin() {
    let path = repo().join("tests/lint/pointer_lifetime/strict/temporary-buffer-result.jai");
    let (lints, text) = lint(&path, &[("temp_storage_escape", Level::Warn)]);
    assert_eq!(lints.len(), 1, "{}", rendered(&lints, &text, &path));
    assert!(lints[0].start > text.find("keep ::").unwrap());
    let (uses, text) = lint(&path, &[("use_after_free", Level::Warn)]);
    assert_eq!(uses.len(), 1, "{}", rendered(&uses, &text, &path));
}

#[test]
fn lifetime_owned_array_elements_do_not_borrow_the_container() {
    let path = repo().join("tests/lint/pointer_lifetime/strict/owned-popped-record.jai");
    let (lints, text) = lint(
        &path,
        &[
            ("pointer_lifetime", Level::Warn),
            ("pointer_into_resizable", Level::Warn),
        ],
    );
    assert!(lints.is_empty(), "{}", rendered(&lints, &text, &path));
}

#[test]
fn lifetime_frozen_result_tracks_the_outer_call_after_argument_calls() {
    let path = repo().join("tests/lint/pointer_lifetime/strict/frozen-composite-call.jai");
    let (lints, text) = lint(&path, &[("returned_local_address", Level::Warn)]);
    assert!(lints.is_empty(), "{}", rendered(&lints, &text, &path));
}

#[test]
fn lifetime_freeing_distinct_array_elements_is_not_double_free() {
    let path = repo().join("tests/lint/pointer_lifetime/strict/distinct-array-frees.jai");
    let (lints, text) = lint(&path, &[("double_free", Level::Warn)]);
    assert_eq!(lints.len(), 1, "{}", rendered(&lints, &text, &path));
    assert!(lints[0].start > text.find("same_element ::").unwrap());
}

#[test]
fn lifetime_variadic_growth_is_not_unconditionally_an_invalidation() {
    let path = repo().join("tests/lint/pointer_lifetime/strict/variadic-growth.jai");
    let rules = [
        ("pointer_lifetime", Level::Warn),
        ("pointer_into_resizable", Level::Warn),
    ];
    let (lints, text) = lint(&path, &rules);
    assert!(lints.is_empty(), "{}", rendered(&lints, &text, &path));
}

#[test]
fn lifetime_pointer_field_loads_do_not_use_future_writes() {
    let path = repo().join("tests/lint/pointer_lifetime/strict/pointer-field-write-order.jai");
    let (lints, text) = lint(
        &path,
        &[
            ("temp_storage_escape", Level::Warn),
            ("returned_local_address", Level::Warn),
        ],
    );
    assert!(lints.is_empty(), "{}", rendered(&lints, &text, &path));
}

#[test]
fn lifetime_ci_online_snapshot() {
    let path = repo().join("tests/lint/pointer_lifetime/strict/ci-online-snapshot.jai");
    let (lints, text) = lint(&path, &[("temp_storage_escape", Level::Warn)]);
    assert!(lints.is_empty(), "{}", rendered(&lints, &text, &path));
    let (lints, text) = lint_with(
        &path,
        &[("temp_storage_escape", Level::Warn)],
        Config {
            lifetime_mode: Some(jacket_config::LifetimeMode::Strict),
            ..Config::default()
        },
    );
    assert_eq!(lints.len(), 1, "{}", rendered(&lints, &text, &path));
    assert!(lints[0].start > text.find("keep ::").unwrap());
    let (lints, text) = lint_with(
        &path,
        &[("use_after_free", Level::Warn)],
        Config {
            lifetime_mode: Some(jacket_config::LifetimeMode::Strict),
            ..Config::default()
        },
    );
    assert_eq!(lints.len(), 1, "{}", rendered(&lints, &text, &path));
}

#[test]
fn lifetime_ci_environment_fiber() {
    let path = repo().join("tests/lint/pointer_lifetime/strict/ci-environment-fiber.jai");
    let (lints, text) = lint(&path, &[("temp_storage_escape", Level::Warn)]);
    assert_eq!(lints.len(), 1, "{}", rendered(&lints, &text, &path));
    assert!(lints[0].start > text.find("bad_saved =").unwrap());
}
