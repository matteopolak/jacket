//! Local headers retain definition-site storage recipes, not sampled values.
use jai_modules::{ModuleGraph, SourceOverlay};
use jai_vm::{Limits, Outcome, Value};
use std::path::Path;

fn graph(source: &str) -> ModuleGraph {
    let path = Path::new("/jai-local-runtime-defaults/main.jai");
    let mut overlay = SourceOverlay::new();
    overlay.insert(path, source.as_bytes().to_vec()).unwrap();
    ModuleGraph::load_with_provider(path, Default::default(), &overlay).unwrap()
}

fn run(source: &str) -> i128 {
    let graph = graph(source);
    let program = jai_sema::resolve_graph(&graph).unwrap();
    let outcome = jai_vm::execute(&program, Limits::default()).outcome;
    let Outcome::Complete(values) = outcome else {
        panic!("runtime default fixture did not complete: {outcome:?}");
    };
    let [Value::Int(value)] = values.as_slice() else {
        panic!("expected one integer result: {values:?}");
    };
    value.value()
}

#[test]
fn nested_procedure_reads_the_current_defining_global_after_caller_shadowing() {
    assert_eq!(
        run(
            "state:int=3;main::()->int{read::(value:int=state)->int{return value;}state=42;{state::99;return read();}}"
        ),
        42
    );
}

#[test]
fn local_record_method_and_alias_keep_the_same_global_default_recipe() {
    assert_eq!(
        run(
            "state:int=3;main::()->int{R::struct{read::(value:int=state)->int{return value;}}Alias::R.read;state=21;{state::99;return R.read()+Alias();}}"
        ),
        42
    );
}

#[test]
fn promoted_global_field_default_keeps_its_canonical_storage_projection() {
    assert_eq!(
        run(
            "Base::struct{value:int=1;}Box::struct{using base:Base;}storage:Box;main::()->int{read::(value:int=storage.value)->int{return value;}storage.base.value=42;{storage::99;return read();}}"
        ),
        42
    );
}

#[test]
fn local_record_method_default_reads_the_callers_actual_implicit_context() {
    assert_eq!(
        run(
            "#add_context marker:int=7;main::()->int{R::struct{read::(value:int=context.marker)->int{return value;}}Alias::R.read;context.marker=21;return R.read()+Alias();}"
        ),
        42
    );
}

#[test]
fn inferred_global_default_retains_a_ready_type_without_sampling_the_value() {
    assert_eq!(
        run(
            "state:int=3;main::()->int{read::(value:=state)->int{return value;}state=42;return read();}"
        ),
        42
    );
}

#[test]
fn lexical_constant_shadow_remains_a_constant_default() {
    assert_eq!(
        run(
            "STATE:int=3;main::()->int{STATE::42;read::(value:int=STATE)->int{return value;}return read();}"
        ),
        42
    );
}

#[test]
fn runtime_local_shadow_cannot_be_replaced_by_an_equal_spelled_global() {
    let source =
        "captured:int=7;main::(){captured:=42;read::(value:int=captured)->int{return value;}}";
    let graph = graph(source);
    let error = jai_sema::resolve_graph(&graph).unwrap_err();
    assert!(error.message.contains("capture"), "{error}");
    assert_eq!(error.location.span.text(source), "captured");
    assert_eq!(error.location.span.start, source.rfind("captured").unwrap());
}
