use super::*;
use jai_modules::{GraphOptions, ModuleGraph};
use std::{
    fs,
    sync::atomic::{AtomicUsize, Ordering},
};

fn resolve_source(source: &str) -> Result<Program, String> {
    static NEXT: AtomicUsize = AtomicUsize::new(0);
    struct Fixture(std::path::PathBuf);
    impl Drop for Fixture {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }
    let fixture = Fixture(std::env::temp_dir().join(format!(
        "jai-pointer-sema-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    )));
    fs::create_dir_all(&fixture.0).unwrap();
    let path = fixture.0.join("main.jai");
    fs::write(&path, source).unwrap();
    let graph =
        ModuleGraph::load(&path, GraphOptions::default()).map_err(|error| error.to_string())?;
    resolve_graph(&graph).map_err(|error| error.to_string())
}
fn execute(source: &str) -> i128 {
    let program = resolve_source(source).unwrap();
    let outcome = jai_vm::execute(&program, jai_vm::Limits::default()).outcome;
    let jai_vm::Outcome::Complete(values) = outcome else {
        panic!("{outcome:?}")
    };
    values[0].integer().unwrap().value()
}

#[test]
fn address_alias_mutation_and_pointer_parameter() {
    assert_eq!(
        execute(
            "set :: (p:*int) { p.* = 42; } main :: ()->int { x := 1; p := *x; set(p); return x; }"
        ),
        42
    );
}

#[test]
fn null_equality_and_short_circuit_do_not_dereference() {
    assert_eq!(
        execute(
            "main :: ()->int { p:*int = null; if p != null && p.* == 4 return 1; if !p || p.* == 4 return 42; return 0; }"
        ),
        42
    );
    assert_eq!(
        execute(
            "main :: ()->int { x := 42; p:*int = ifx true then *x else null; q:*int = ifx false then null else ifx true then p else null; return q.*; }"
        ),
        42
    );
}

#[test]
fn nested_field_and_global_addresses_preserve_storage_identity() {
    assert_eq!(
        execute(
            "Inner :: struct { x:int; } Outer :: struct { inner:Inner; } global := 3; main :: ()->int { item:Outer; p := *item.inner.x; p.* = 35; q := *global; q.* += 4; return item.inner.x + global; }"
        ),
        42
    );
}

#[test]
fn array_element_address_and_element_stride() {
    assert_eq!(
        execute(
            "main :: ()->int { a:[3]int = .[11,20,31]; p := *a[0]; q := p + 1; q.* += 2; return a[1] + (p + 2).* - 11; }"
        ),
        42
    );
}

#[test]
fn null_needs_a_pointer_context_and_explicit_address_casts_are_typed() {
    assert!(
        resolve_source("main :: () { p := null; }")
            .unwrap_err()
            .contains("null")
    );
    resolve_source("main :: () { x := 1; p := *x; q := cast(*u8) p; }").unwrap();
    assert_eq!(
        execute(
            "main :: ()->int { p:*int = null; address := cast(u64) p; restored := cast(*int) address; if restored == null return 42; return 0; }"
        ),
        42
    );
}

#[test]
fn void_erasure_preserves_pointer_identity() {
    assert_eq!(
        execute(
            "main :: ()->int { x := 42; p := *x; erased := cast(*void) p; if erased != null && p == *x return p.*; return 0; }"
        ),
        42
    );
    assert_eq!(
        execute(
            "main :: ()->int { x := 42; bytes := cast(*u8) *x; restored := cast(*int) bytes; return restored.*; }"
        ),
        42
    );
    assert_eq!(
        execute(
            "main :: ()->int { x := 42; erased := cast(*void) *x; restored := cast,no_check(*int) erased; return restored.*; }"
        ),
        42
    );
}

#[test]
fn indirect_access_checks_null_bounds_and_released_frames() {
    let program = resolve_source("main :: ()->int { p:*int = null; return p.*; }").unwrap();
    assert_eq!(
        jai_vm::execute(&program, jai_vm::Limits::default()).outcome,
        jai_vm::Outcome::Failed(jai_vm::Error::NullPointer)
    );
    let program =
        resolve_source("main :: ()->int { a:[1]int = .[42]; index := 1; return a[index]; }")
            .unwrap();
    assert!(matches!(
        jai_vm::execute(&program, jai_vm::Limits::default()).outcome,
        jai_vm::Outcome::Failed(jai_vm::Error::OutOfBounds { .. })
    ));
    let program = resolve_source(
        "escape :: ()->*int { x := 42; return *x; } main :: ()->int { p := escape(); return p.*; }",
    )
    .unwrap();
    assert_eq!(
        jai_vm::execute(&program, jai_vm::Limits::default()).outcome,
        jai_vm::Outcome::Failed(jai_vm::Error::DanglingPointer)
    );
}

#[test]
fn builtin_types_are_values_after_lexical_name_lookup() {
    assert_eq!(
        execute(
            "main :: ()->int { value := 42; if type_of(*value) == *int && int == s64 && float == float32 && u8 != u64 && bool != void && *#Context != *void return value; return 0; }"
        ),
        42
    );
    assert_eq!(
        execute(
            "main :: ()->int { int := 40; float32 := 2; p := *int; p.* += float32; return int; }"
        ),
        42
    );
}

#[test]
fn pointer_integer_roundtrip_and_relative_addresses_retain_identity() {
    assert_eq!(
        execute(
            "main :: ()->int { value := 1; p := *value; address := cast(u64) p; restored := cast(*int) address; restored.* = 42; return value; }"
        ),
        42
    );
    assert_eq!(
        execute(
            "main :: ()->int { values:[4]u16; base := cast(u64) *values[0]; last := cast(u64) *values[3]; return cast(int)(last-base) + 36; }"
        ),
        42
    );
}

#[test]
fn integer_left_pointer_offset_preserves_operand_order() {
    assert_eq!(
        execute(
            "calls := 0; offset :: ()->int { calls = 1; return 1; } pointer :: (p:*int)->*int { if calls == 1 calls = 2; else calls = 99; return p; } main :: ()->int { values:[2]int = .[0,40]; p := offset() + pointer(*values[0]); p.* += calls; return values[1]; }"
        ),
        42
    );
}
