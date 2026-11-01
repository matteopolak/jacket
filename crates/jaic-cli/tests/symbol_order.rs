//! Output that a user can see must not follow symbol numbers, which depend on which parse thread
//! met a spelling first. Each program is compiled under several `JAIC_SYM_SCRAMBLE` seeds and the
//! output compared (`test_support::determinism`).
use std::path::{Path, PathBuf};
use test_support::determinism::assert_stable;

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn fixture(name: &str) -> PathBuf {
    root().join("tests/fixtures/symbol-order").join(name)
}

fn jaic() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_jaic"))
}

/// Checks `program` and requires a failure whose output holds `first`.
fn first_error(program: &Path, first: &str) {
    let output = assert_stable(&jaic(), program, &["-check"]);
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains(first), "{stderr}");
}

#[test]
fn the_first_reported_overload_set_is_the_first_declared() {
    first_error(&fixture("overloads.jai"), "overloads.jai:4:1: error");
}

#[test]
fn the_first_reported_using_clash_is_the_first_declared() {
    first_error(&fixture("using-clash.jai"), "`zulu` is already declared");
}

#[test]
fn namespace_suggestions_come_in_declaration_order() {
    let output = assert_stable(&jaic(), &fixture("namespaces.jai"), &["-check"]);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let at = |text: &str| {
        stderr
            .find(text)
            .unwrap_or_else(|| panic!("{text}\n{stderr}"))
    };
    assert!(at("Zed.min") < at("Alpha.min"), "{stderr}");
    assert!(at("Alpha.min") < at("Yak.min"), "{stderr}");
}

#[test]
fn several_errors_read_the_same_under_every_seed() {
    let output = assert_stable(&jaic(), &fixture("many-errors.jai"), &["-check"]);
    assert!(!output.status.success());
}

#[test]
fn the_type_table_lists_records_in_declaration_order_under_every_seed() {
    let program = root().join("stdlib/Compiler/tests/type-table-declaration-order.jai");
    let output = assert_stable(&jaic(), &program, &["-interp"]);
    assert_eq!(String::from_utf8_lossy(&output.stdout), "ok\n");
}

#[test]
fn a_metaprogramming_example_checks_the_same_under_every_seed() {
    let program = root().join("examples/metaprogramming/main.jai");
    let output = assert_stable(&jaic(), &program, &["-check"]);
    assert!(output.status.success());
}
