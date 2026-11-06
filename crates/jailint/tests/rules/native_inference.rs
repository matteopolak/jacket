//! Checked native and captured-return consumer regressions.
use super::*;

#[test]
fn unpaired_native_locks_infer_effects_without_inventing_wrapper_intent() {
    let path = repo().join("tests/lint/unpaired_lock/native-inference.jai");
    let mut options = jaic::sema::Options::host();
    options.os = jaic::target::TargetOs::Windows;
    options.cpu = jaic::target::TargetCpu::X64;
    let (lints, text) = lint_target(
        &path,
        &[("unpaired_lock", Level::Warn)],
        Config::default(),
        options,
    );
    assert_eq!(lints.len(), 10, "{}", rendered(&lints, &text, &path));
    for name in [
        "ordinary_bug",
        "nested_bug",
        "nested_defer_bug",
        "wrong_internal_receiver",
        "inferred_spelling_is_not_fix_intent",
        "reordered_named_binding_bug",
        "second_formal_binding_bug",
        "retargeted_receiver_bug",
        "conditional_release_bug",
        "discharged_then_reacquired_bug",
    ] {
        let start = text.find(&format!("{name} ::")).unwrap();
        let end = start + text[start..].find("\n}").unwrap() + 2;
        assert!(
            lints.iter().any(|l| l.start >= start && l.start < end),
            "missing {name}: {}",
            rendered(&lints, &text, &path)
        );
    }
    let discharged = text
        .find("raw_enter(p); // this obligation is discharged")
        .unwrap();
    let reacquired = text
        .find("raw_enter(p); // only this later acquisition leaks")
        .unwrap();
    assert!(
        !lints.iter().any(|l| l.start == discharged),
        "an already discharged obligation cannot be reacquired retroactively"
    );
    assert!(lints.iter().any(|l| l.start == reacquired));
    let leaked_first = text.find("raw_enter(holder.first); // this first").unwrap();
    assert!(
        lints.iter().any(|l| l.start == leaked_first),
        "retargeting must report the unreleased first object: {}",
        rendered(&lints, &text, &path)
    );
    assert!(
        lints.iter().all(|l| l.fix.is_none()),
        "new native inference must not guess an unlock spelling"
    );
}

#[test]
fn unpaired_windows_lock_release_preserves_existing_diagnostics() {
    let mut options = jaic::sema::Options::host();
    options.os = jaic::target::TargetOs::Windows;
    options.cpu = jaic::target::TargetCpu::X64;
    for (name, count) in [("bad.jai", 2), ("good.jai", 0)] {
        let path = repo().join("tests/lint/unpaired_lock").join(name);
        let (lints, text) = lint_target(
            &path,
            &[("unpaired_lock", Level::Warn)],
            Config::default(),
            options.clone(),
        );
        assert_eq!(lints.len(), count, "{}", rendered(&lints, &text, &path));
    }
}

#[test]
fn unclosed_resources_bind_the_declared_receiver_before_classifying_mentions() {
    let path = repo().join("tests/lint/unclosed_resource/named-bindings.jai");
    let (lints, text) = lint(&path, &[("unclosed_resource", Level::Warn)]);
    assert_eq!(lints.len(), 1, "{}", rendered(&lints, &text, &path));
    assert!(
        lints[0].message.contains("`b`"),
        "{}",
        rendered(&lints, &text, &path)
    );
    assert!(lints[0].start < text.find("passed_on_as_other").unwrap());
}

#[test]
fn native_lock_candidate_fanout_exhaustion_does_not_publish_partial_reports() {
    let root = TempDir::new("jailint-native-lock-budget");
    let file = root.path().join("main.jai");
    let source = format!(
        "lib :: #library,system \"kernel32\";\nenter :: (p:*void) #foreign lib \"EnterCriticalSection\";\nprobe :: (p:*void) {{\n{}\n}}\nmain :: () {{}}\n",
        "enter(p);\n".repeat(300)
    );
    std::fs::write(&file, source).unwrap();
    let mut options = jaic::sema::Options::host();
    options.os = jaic::target::TargetOs::Windows;
    options.cpu = jaic::target::TargetCpu::X64;
    let (lints, _) = lint_target(
        &file,
        &[("unpaired_lock", Level::Warn)],
        Config::default(),
        options,
    );
    assert!(
        lints.is_empty(),
        "bounded native analysis must remain unknown as a whole: {lints:?}"
    );
}

#[test]
fn lifetime_captured_read_returns_require_current_selector_proof() {
    let path = repo().join("tests/lint/pointer_lifetime/resources/captured-read-returns.jai");
    for mode in [
        jacket_config::LifetimeMode::Must,
        jacket_config::LifetimeMode::Strict,
    ] {
        let (lints, text) = lint_with(
            &path,
            &[("use_after_free", Level::Warn)],
            Config {
                lifetime_mode: Some(mode),
                ..Config::default()
            },
        );
        assert_eq!(lints.len(), 2, "{}", rendered(&lints, &text, &path));
        for name in ["pure_projection", "cast_projection"] {
            let start = text.find(&format!("{name} ::")).unwrap();
            let end = start + text[start..].find("\n}").unwrap() + 2;
            assert!(
                lints.iter().any(|l| l.start >= start && l.start < end),
                "missing {name}: {}",
                rendered(&lints, &text, &path)
            );
        }
    }
}

#[test]
fn unclosed_resources_use_checked_native_returns_without_opening_notes() {
    let path = repo().join("tests/lint/unclosed_resource/native-inferred.jai");
    for target in [
        jaic::target::TargetOs::Linux,
        jaic::target::TargetOs::Windows,
    ] {
        let mut options = jaic::sema::Options::host();
        options.os = target;
        options.cpu = jaic::target::TargetCpu::X64;
        let (lints, text) = lint_target(
            &path,
            &[("unclosed_resource", Level::Warn)],
            Config::default(),
            options,
        );
        assert_eq!(
            lints.len(),
            8,
            "{target:?}: {}",
            rendered(&lints, &text, &path)
        );
        let start = text.find("leaked_annotated_both ::").unwrap();
        let end = start + text[start..].find('\n').unwrap();
        let annotated: Vec<_> = lints
            .iter()
            .filter(|lint| lint.start >= start && lint.start < end)
            .collect();
        assert_eq!(
            annotated.len(),
            2,
            "both returned acquisitions must be retained"
        );
        for name in ["first", "second"] {
            assert!(
                annotated
                    .iter()
                    .any(|lint| lint.message.contains(&format!("`{name}`")))
            );
        }
        let closed_start = text.find("closed_one ::").unwrap();
        let second = closed_start + text[closed_start..].find("second").unwrap();
        let closed_end = closed_start + text[closed_start..].find('\n').unwrap();
        let closed: Vec<_> = lints
            .iter()
            .filter(|lint| lint.start >= closed_start && lint.start < closed_end)
            .collect();
        assert_eq!(
            closed.len(),
            1,
            "closing first must retain only second's obligation"
        );
        assert_eq!(
            closed[0].start, second,
            "the remaining resource binding is the diagnostic location"
        );
        for name in [
            "leaked_first",
            "leaked_second",
            "leaked_before_callback",
            "leaked_named",
            "leaked_returns_other",
        ] {
            let start = text.find(&format!("{name} ::")).unwrap();
            let end = start + text[start..].find('\n').unwrap_or(text.len() - start);
            assert!(
                lints
                    .iter()
                    .any(|lint| lint.start >= start && lint.start < end),
                "missing {name}: {}",
                rendered(&lints, &text, &path)
            );
        }
    }
}

#[test]
fn inferred_unclosed_resources_respect_unmentioned_global_and_alias_releases() {
    let path = repo().join("tests/lint/unclosed_resource/native-global-release.jai");
    let mut options = jaic::sema::Options::host();
    options.os = jaic::target::TargetOs::Linux;
    options.cpu = jaic::target::TargetCpu::X64;
    let (lints, text) = lint_target(
        &path,
        &[("unclosed_resource", Level::Warn)],
        Config::default(),
        options,
    );
    assert!(lints.is_empty(), "{}", rendered(&lints, &text, &path));
}
