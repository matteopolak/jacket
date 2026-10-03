use super::*;
use jai_modules::GraphOptions;
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
        "jai-variants-{}-{}",
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
        panic!("{outcome:?}");
    };
    values[0].integer().unwrap().value()
}
#[test]
fn literals_and_numeric_operations_retain_the_distinct_identity() {
    assert_eq!(
        execute(
            "Handle :: #type,distinct u32; main :: () -> int { a: Handle = 5; b: Handle = 3 * a + 2; return cast(int) b; }"
        ),
        17
    );
}
#[test]
fn explicit_cast_and_transparent_alias_reuse_the_nominal_type() {
    assert_eq!(
        execute(
            "Handle :: #type,distinct u32; Alias :: Handle; main :: () -> int { plain: u32 = 9; a: Handle = cast(Handle) plain; b: Alias = a; return cast(int) b; }"
        ),
        9
    );
}
#[test]
fn isa_chain_converts_to_ancestors_without_allowing_sibling_conversion() {
    assert_eq!(
        execute(
            "Base :: #type,distinct u32; Child :: #type,isa Base; Grandchild :: #type,isa Child; consume :: (value: Base) -> int { return cast(int) value; } main :: () -> int { value: Grandchild = 13; return consume(value); }"
        ),
        13
    );
}
#[test]
fn nonliteral_and_sibling_values_require_explicit_casts() {
    for source in [
        "Handle :: #type,distinct u32; main :: () { value: u32 = 5; handle: Handle = value; }",
        "A :: #type,distinct u32; B :: #type,distinct u32; main :: () { a: A = 5; b: B = a; }",
        "Base :: #type,distinct u32; Child :: #type,isa Base; main :: () { base: Base = 5; child: Child = base; }",
    ] {
        assert!(
            resolve_source(source)
                .unwrap_err()
                .contains("explicit cast"),
            "{source}"
        );
    }
}
#[test]
fn isa_scalar_base_can_continue_through_range_preserving_widening() {
    assert_eq!(
        execute(
            "Count :: #type,isa u32; consume :: (value: int) -> int { return value; } main :: () -> int { value: Count = 11; return consume(value); }"
        ),
        11
    );
}
