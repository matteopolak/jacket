//! Member defaults of a struct nothing instantiates are still checked.
use test_support::compiler::compile_virtual;

fn check(source: &str) -> Result<(), String> {
    compile_virtual(&[("/main.jai", source)], "/main.jai", |_| {})
        .1
        .map_err(|d| d.message)
}

#[test]
fn a_default_of_the_wrong_type_is_an_error() {
    let message = check("Unused :: struct { n: s64 = \"text\"; }\nmain :: () {}\n").unwrap_err();

    assert!(message.contains("s64"), "{message}");
}

#[test]
fn a_default_naming_nothing_is_an_error() {
    let message = check("Unused :: struct { n: s64 = missing; }\nmain :: () {}\n").unwrap_err();

    assert!(message.contains("missing"), "{message}");
}

#[test]
fn an_unused_struct_with_good_defaults_compiles() {
    check("Unused :: struct { n: s64 = 4; f: float = 1.5; v: [2] u8 = .[1, 2]; }\nmain :: () {}\n")
        .unwrap();
}

#[test]
fn a_polymorphic_struct_is_checked_per_instance() {
    check("Box :: struct (T: Type) { n: T = \"text\"; }\nmain :: () {}\n").unwrap();
}
