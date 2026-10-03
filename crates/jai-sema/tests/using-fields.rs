use jai_modules::{GraphOptions, ModuleGraph};
use jai_sema::resolve_graph;
use jai_vm::{Limits, Outcome, Value};
use std::{
    path::PathBuf,
    sync::atomic::{AtomicUsize, Ordering},
};

struct Fixture(PathBuf);

impl Fixture {
    fn new(source: &str) -> Self {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let root = std::env::temp_dir().join(format!(
            "jai-using-fields-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir_all(&root).unwrap();
        std::fs::write(root.join("main.jai"), source).unwrap();
        Self(root)
    }

    fn graph(&self) -> ModuleGraph {
        ModuleGraph::load(&self.0.join("main.jai"), GraphOptions::default()).unwrap()
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn run(source: &str) -> i128 {
    let fixture = Fixture::new(source);
    let program = resolve_graph(&fixture.graph())
        .unwrap_or_else(|error| panic!("using field source must resolve: {error:?}"));
    let execution = jai_vm::execute(&program, Limits::default());
    let Outcome::Complete(values) = execution.outcome else {
        panic!("using field source must execute: {execution:?}");
    };
    let [Value::Int(result)] = values.as_slice() else {
        panic!("expected one integer result: {values:?}");
    };
    result.value()
}

fn reject(source: &str, expected: &str) {
    let fixture = Fixture::new(source);
    let error = resolve_graph(&fixture.graph()).unwrap_err();
    assert!(
        error.message.contains(expected),
        "expected {expected:?}, got {error:?}"
    );
}

#[test]
fn nested_promotion_mutates_the_complete_owned_storage_path() {
    assert_eq!(
        run(r#"
Outer :: struct { using middle: Middle; }
Middle :: struct { using inner: Inner; }
Inner :: struct { value: int = 7; }
main :: () -> int {
    outer: Outer;
    outer.value += 35;
    return outer.middle.inner.value;
}
"#),
        42
    );
}

#[test]
fn direct_member_conflicts_with_a_promoted_declaration() {
    reject(
        r#"
Inner :: struct { value: int; }
Outer :: struct { using inner: Inner; value: u32; }
main :: () {}
"#,
        "ambiguous",
    );
}

#[test]
fn diamond_is_ambiguous_even_when_leaf_identity_is_shared() {
    reject(
        r#"
Inner :: struct { value: int; }
Outer :: struct { using a: Inner; using b: Inner; }
main :: () {}
"#,
        "ambiguous",
    );
}

#[test]
fn nonrecord_and_cyclic_using_edges_are_rejected() {
    for declarations in [
        "A :: struct { using count: int; }",
        "A :: struct { using b: B; } B :: struct { using a: A; }",
    ] {
        reject(&format!("{declarations} main :: () {{}}"), "using field");
    }
}
