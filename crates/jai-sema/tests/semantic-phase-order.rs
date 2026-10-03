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
            "jai-semantic-phase-order-{}-{}",
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
        .unwrap_or_else(|error| panic!("semantic phase source must resolve: {error:?}"));
    let execution = jai_vm::execute(&program, Limits::default());
    let Outcome::Complete(values) = execution.outcome else {
        panic!("semantic phase source must execute: {execution:?}");
    };
    let [Value::Int(result)] = values.as_slice() else {
        panic!("expected one integer result: {values:?}");
    };
    result.value()
}

#[test]
fn ordinary_using_parameter_names_are_bound_after_header_reservation() {
    assert_eq!(
        run(r#"
Record :: struct { value: int; }
read :: (using record: Record) -> int { return value; }
main :: () -> int { value: Record = .{value=42}; return read(value); }
"#),
        42
    );
}

#[test]
fn context_record_field_defaults_wait_for_the_defined_schema() {
    for field in ["saved: #Context;", "saved: #Context = .{};"] {
        assert_eq!(
            run(&format!(
                "#add_context number: int = 40; Holder :: struct {{ {field} }} \
                 main :: () -> int {{ holder: Holder; return holder.saved.number + 2; }}"
            )),
            42
        );
    }
}
