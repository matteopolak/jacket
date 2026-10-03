use jai_types::LayoutPolicy;
use std::{
    path::PathBuf,
    sync::atomic::{AtomicUsize, Ordering},
};

struct Fixture(PathBuf);
impl Fixture {
    fn new(source: &str) -> Self {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let path = std::env::temp_dir().join(format!(
            "jai-runtime-type-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir_all(&path).unwrap();
        std::fs::write(path.join("main.jai"), source).unwrap();
        Self(path)
    }
    fn program(&self) -> Result<jai_ir::Program, String> {
        let graph = jai_modules::ModuleGraph::load(
            &self.0.join("main.jai"),
            jai_modules::GraphOptions::default(),
        )
        .map_err(|error| error.to_string())?;
        jai_sema::resolve_graph_with_options(
            &graph,
            &jai_sema::ResolveOptions {
                layout: Some(LayoutPolicy::lp64()),
                ..Default::default()
            },
            &mut jai_vm::NoEffects,
        )
        .map_err(|error| error.to_string())
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}
fn check(source: &str) {
    let fixture = Fixture::new(source);
    let program = fixture.program().unwrap();
    let outcome = jai_vm::execute(&program, jai_vm::Limits::default()).outcome;
    let jai_vm::Outcome::Complete(values) = outcome else {
        panic!("runtime Type source did not complete: {outcome:?}");
    };
    assert_eq!(values[0].integer().unwrap().value(), 42);
}
#[test]
fn runtime_types_pass_return_copy_and_default_to_null() {
    check(include_str!("fixtures/runtime-type-basics.jai"));
}
#[test]
fn type_record_fields_arrays_and_addresses_keep_nominal_identity() {
    check(include_str!("fixtures/runtime-type-storage.jai"));
}
#[test]
fn descriptor_casts_reuse_exact_canonical_cyclic_reflection_objects() {
    check(include_str!("fixtures/runtime-type-descriptors.jai"));
}
#[test]
fn conditional_types_preserve_selected_values_and_null_default() {
    check(include_str!("fixtures/runtime-type-conditionals.jai"));
}
#[test]
fn c_signature_type_parameters_results_and_callback_cells_are_pointers() {
    check(include_str!("fixtures/runtime-type-c-calls.jai"));
}
#[test]
fn compile_time_type_results_publish_certified_descriptor_relocations() {
    check(include_str!("fixtures/runtime-type-run.jai"));
}
#[test]
fn global_type_initializers_emit_real_descriptor_relocations() {
    check(include_str!("fixtures/runtime-type-globals.jai"));
}
#[test]
fn type_runtime_operators_do_not_decay_to_pointer_arithmetic() {
    let fixture = Fixture::new("main::()->int{t:Type=s32;t+=1;return 0;}");
    assert!(
        fixture
            .program()
            .unwrap_err()
            .contains("Type values only support nominal equality")
    );
}
