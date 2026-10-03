//! Independent source fixtures exercise bootstrap registration before schema freezing.
use jai_modules::{
    BootstrapOptions, GraphOptions, ModuleGraph, PreludeSource, RuntimeSupportOptions,
    RuntimeSupportParameters, RuntimeSupportSource, SourceOverlay,
};
use jai_sema::resolve_graph;
use jai_vm::{Limits, Outcome, Value};
use std::path::Path;

fn graph(prelude: &str, application: &str) -> ModuleGraph {
    let prelude = format!(
        "{}\n{prelude}",
        include_str!("../../../stdlib/prelude/module.jai")
    );
    let mut sources = SourceOverlay::new();
    for (path, source) in [
        ("/jai-context-bootstrap/main.jai", application),
        (
            "/jai-context-bootstrap/modules/module.jai",
            prelude.as_str(),
        ),
        (
            "/jai-context-bootstrap/modules/Runtime_Support.jai",
            "#module_parameters(DEFINE_SYSTEM_ENTRY_POINT:bool, DEFINE_INITIALIZATION:bool, ENABLE_BACKTRACE_ON_CRASH:bool); Context_Base::struct{value:int=7;}",
        ),
    ] {
        sources
            .insert(Path::new(path), source.as_bytes().to_vec())
            .unwrap();
    }
    ModuleGraph::load_with_bootstrap_options(
        Path::new("/jai-context-bootstrap/main.jai"),
        GraphOptions {
            import_dirs: vec!["/jai-context-bootstrap/modules".into()],
        },
        BootstrapOptions {
            prelude: PreludeSource::Search,
            runtime_support: Some(RuntimeSupportOptions {
                source: RuntimeSupportSource::Search,
                parameters: RuntimeSupportParameters {
                    define_system_entry_point: false,
                    define_initialization: false,
                    enable_backtrace_on_crash: false,
                },
            }),
        },
        &sources,
        None,
    )
    .unwrap()
}

#[test]
fn designated_preload_quote_registers_first_with_its_defining_scope() {
    let graph = graph(
        "FIRST_ADD_CONTEXT::#code #add_context #as using base:Context_Base;",
        "FIRST_ADD_CONTEXT::3; Context_Base::struct{wrong:int;} #add_context extra:int=5; main::()->int{return context.base.value+context.extra;}",
    );
    let program = resolve_graph(&graph).unwrap();
    let definition = program.context().unwrap();
    let jai_ir::ConstantKind::Record(fields) = &definition.default.kind else {
        panic!("context default must be a typed record");
    };
    assert_eq!(fields.len(), 2);
    assert!(matches!(fields[0].kind, jai_ir::ConstantKind::Record(_)));
    assert!(matches!(&fields[1].kind, jai_ir::ConstantKind::Int(value) if value.value()==5));
    assert!(
        matches!(jai_vm::execute(&program, Limits::default()).outcome,
        Outcome::Complete(values) if matches!(values.as_slice(), [Value::Int(value)] if value.value()==12))
    );
}

#[test]
fn bootstrap_quote_duplicate_registration_is_source_located() {
    let graph = graph(
        "FIRST_ADD_CONTEXT::#code #add_context #as using base:Context_Base;",
        "#add_context base:int; main::(){}",
    );
    let error = resolve_graph(&graph).unwrap_err();
    assert!(
        error.message.contains("conflicting context member name"),
        "{error}"
    );
    assert_eq!(
        graph.sources().get(error.location.source).unwrap().path(),
        Path::new("/jai-context-bootstrap/main.jai")
    );
}

#[test]
fn bootstrap_does_not_execute_arbitrary_quoted_statements() {
    let graph = graph(
        "FIRST_ADD_CONTEXT::#code { #add_context base:Context_Base; value:=7; };",
        "main::(){}",
    );
    let error = resolve_graph(&graph).unwrap_err();
    assert!(
        error.message.contains("only structural #add_context"),
        "{error}"
    );
    assert_eq!(
        graph.sources().get(error.location.source).unwrap().path(),
        Path::new("/jai-context-bootstrap/modules/module.jai")
    );
}

#[test]
fn bootstrap_using_fields_support_writes_and_scoped_call_overrides() {
    let graph = graph(
        "FIRST_ADD_CONTEXT::#code #add_context #as using base:Context_Base;",
        "read::()->int{return context.value;} main::()->int{context.value=11; return read(,,value=42)+context.value;}",
    );
    let program = resolve_graph(&graph).unwrap();
    assert!(
        matches!(jai_vm::execute(&program, Limits::default()).outcome,
        Outcome::Complete(values) if matches!(values.as_slice(), [Value::Int(value)] if value.value()==53))
    );
}

#[test]
fn inactive_bootstrap_branches_do_not_register_context_fields() {
    let graph = graph(
        "FIRST_ADD_CONTEXT::#code { #if true { #add_context #as using base:Context_Base; } else { #add_context base:Missing_Type; } };",
        "main::()->int{return context.value;}",
    );
    let program = resolve_graph(&graph).unwrap();
    assert!(
        matches!(jai_vm::execute(&program, Limits::default()).outcome,
        Outcome::Complete(values) if matches!(values.as_slice(), [Value::Int(value)] if value.value()==7))
    );
}

#[test]
fn inferred_default_waits_for_the_selected_bootstrap_context() {
    let graph = graph(
        "FIRST_ADD_CONTEXT::#code #add_context #as using base:Context_Base;",
        "read::(value:=context.value)->int{return value;} main::()->int{context.value=42;return read();}",
    );
    let program = resolve_graph(&graph).unwrap();
    let read = graph
        .declarations()
        .iter()
        .find(|declaration| graph.symbols().name(declaration.name()) == "read")
        .unwrap();
    let signature = program.library().procedure(read.id()).unwrap();
    let context_type = program.context().unwrap().record_type;
    let base = program.library().types().field(context_type, 0).unwrap().ty;
    let field = program.library().types().field(base, 0).unwrap().ty;
    assert_eq!(signature.parameters[0].ty(), field);
    assert!(
        matches!(jai_vm::execute(&program, Limits::default()).outcome,
        Outcome::Complete(values) if matches!(values.as_slice(), [Value::Int(value)] if value.value()==42))
    );
}
