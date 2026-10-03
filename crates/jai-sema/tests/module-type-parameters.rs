//! Parameterized source instances share the semantic type registry and execute their typed bodies.
use jai_modules::{GraphOptions, ModuleGraph, SourceOverlay};
use jai_vm::{Limits, Outcome, Value};
use std::path::Path;
fn build_graph(application: &str, module: &str) -> ModuleGraph {
    let mut source = SourceOverlay::new();
    for (path, text) in [
        ("/jai-semantic-types/main.jai", application),
        ("/jai-semantic-types/library.jai", module),
    ] {
        source
            .insert(Path::new(path), text.as_bytes().to_vec())
            .unwrap();
    }
    ModuleGraph::load_with_provider(
        Path::new("/jai-semantic-types/main.jai"),
        GraphOptions::default(),
        &source,
    )
    .unwrap()
}
fn resolve(application: &str, module: &str) -> jai_ir::Program {
    jai_sema::resolve_graph(&build_graph(application, module)).unwrap()
}
#[test]
fn type_arguments_change_typed_procedure_bodies_in_each_instance() {
    let program = resolve(
        "Small::#import,file \"library.jai\"(T=u8); Wide::#import,file \"library.jai\"(T=int); main::()->int{return cast(int) Small.identity(21)+Wide.identity(21);}",
        "#module_parameters(T:Type=int); Alias::T; identity::(value:Alias)->Alias{return value;}",
    );
    assert!(
        matches!(jai_vm::execute(&program,Limits::default()).outcome, Outcome::Complete(values) if matches!(values.as_slice(),[Value::Int(value)] if value.value()==42))
    );
}
#[test]
fn nominal_type_argument_resolves_fields_in_its_original_module_scope() {
    let program = resolve(
        "Point::struct{value:int;} Lib::#import,file \"library.jai\"(T=Point); main::()->int{p:Point; p.value=42; return Lib.read(p);}",
        "#module_parameters(T:Type=int); read::(value:T)->int{return value.value;}",
    );
    assert!(
        matches!(jai_vm::execute(&program,Limits::default()).outcome, Outcome::Complete(values) if matches!(values.as_slice(),[Value::Int(value)] if value.value()==42))
    );
}
#[test]
fn interface_variable_is_a_real_type_binding_in_module_procedure() {
    let program = resolve(
        "Replacement::struct{value:int;} Lib::#import,file \"library.jai\"()(REPLACEMENT_INTERFACE=Replacement); main::()->int{p:Replacement; p.value=42; return Lib.read(p);}",
        "#module_parameters()(REPLACEMENT_INTERFACE:$I/interface Required=Required){Required::struct{value:int;}} read::(value:$I)->int{return value.value;}",
    );
    assert!(
        matches!(jai_vm::execute(&program,Limits::default()).outcome, Outcome::Complete(values) if matches!(values.as_slice(),[Value::Int(value)] if value.value()==42))
    );
}

#[test]
fn interface_type_variable_rejects_an_unrelated_nominal_record() {
    let graph = build_graph(
        "Replacement::struct{value:int;} Other::struct{value:int;} Lib::#import,file \"library.jai\"()(REPLACEMENT_INTERFACE=Replacement); main::()->int{p:Other; return Lib.read(p);}",
        "#module_parameters()(REPLACEMENT_INTERFACE:$I/interface Required=Required){Required::struct{value:int;}} read::(value:$I)->int{return value.value;}",
    );
    let error = jai_sema::resolve_graph(&graph).unwrap_err();
    assert!(
        error
            .message
            .contains("argument type cannot be implicitly converted to the parameter type"),
        "{error}"
    );
    assert_eq!(
        graph.sources().get(error.location.source).unwrap().path(),
        Path::new("/jai-semantic-types/main.jai")
    );
}
