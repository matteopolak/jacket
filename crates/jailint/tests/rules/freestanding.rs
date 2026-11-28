//! The `freestanding` rule's settings and its target-dependent checks.

use super::*;

#[test]
fn freestanding_frame_limit_comes_from_the_settings() {
    let file = repo().join("tests/lint/freestanding/bad.jai");
    let levels = [("freestanding", Level::Warn)];
    let frames = |limit| {
        let config = Config {
            max_frame_bytes: Some(limit),
            ..Config::default()
        };
        let (lints, _) = lint_with(&file, &levels, config);

        lints
            .into_iter()
            .filter(|l| l.message.contains("of locals on the stack"))
            .collect::<Vec<_>>()
    };
    // `buffer_work` keeps a 4 KiB array in its locals.
    let lints = frames(1024);

    assert_eq!(lints.len(), 1, "{lints:?}");
    assert!(
        lints[0]
            .message
            .contains("`buffer_work` keeps 4 KiB of locals on the stack, past the limit of 1 KiB")
    );
    assert!(frames(4096).is_empty());
}

#[test]
fn freestanding_cpu_checks_wait_for_a_cpu_that_lacks_something() {
    // AArch64, the only freestanding CPU, has double-precision floats and a 64-bit divide, and a
    // hosted target never reports them.
    let file = repo().join("tests/lint/freestanding/cpu.jai");
    let levels = [("freestanding", Level::Warn)];
    let (lints, text) = lint_with(&file, &levels, Config::default());

    assert!(lints.is_empty(), "{}", rendered(&lints, &text, &file));
    let mut options = jaic::sema::Options::host();

    jaic::target::Target::parse("aarch64-none-elf")
        .unwrap()
        .apply(&mut options);
    let (lints, text) = lint_target(&file, &levels, Config::default(), options);

    assert!(lints.is_empty(), "{}", rendered(&lints, &text, &file));
}

#[test]
fn freestanding_reports_the_cortex_m_gaps_in_double_floats_and_wide_division() {
    let file = repo().join("tests/lint/freestanding/cpu.jai");
    let levels = [("freestanding", Level::Warn)];
    let mut options = jaic::sema::Options::host();

    jaic::target::Target::parse("thumbv7em-none-eabihf")
        .unwrap()
        .apply(&mut options);
    let (lints, text) = lint_target(&file, &levels, Config::default(), options);
    let messages: Vec<_> = lints.iter().map(|l| l.message.as_str()).collect();
    let rendered = rendered(&lints, &text, &file);

    assert_eq!(
        messages
            .iter()
            .filter(|m| m.contains("`float64` arithmetic runs in software"))
            .count(),
        2,
        "{rendered}"
    );
    assert_eq!(
        messages
            .iter()
            .filter(|m| m.contains("64-bit division in a loop runs in software"))
            .count(),
        1,
        "{rendered}"
    );
}
