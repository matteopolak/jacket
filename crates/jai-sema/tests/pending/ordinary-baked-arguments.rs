//! Run after the retained #bake_arguments expression producer is activated.
use jai_modules::{GraphOptions, ModuleGraph, SourceOverlay};
use jai_vm::{Limits, Outcome, Value};
use std::path::Path;

fn compile(source: &str) -> Result<jai_ir::Program, String> {
    let path = Path::new("/ordinary-baked/main.jai");
    let mut overlay = SourceOverlay::new();
    overlay.insert(path, source.as_bytes().to_vec()).unwrap();
    let graph = ModuleGraph::load_with_provider(path, GraphOptions::default(), &overlay)
        .map_err(|error| error.to_string())?;
    jai_sema::resolve_graph_with_options(
        &graph,
        &jai_sema::ResolveOptions {
            layout: Some(jai_types::LayoutPolicy::lp64()),
            ..jai_sema::ResolveOptions::default()
        },
        &mut jai_vm::NoEffects,
    )
    .map_err(|error| error.render(graph.sources()))
}

fn run(source: &str) -> (jai_ir::Program, i128) {
    let program = compile(source).unwrap();
    let result = jai_vm::execute(&program, Limits::default());
    let Outcome::Complete(values) = result.outcome else {
        panic!("{result:?}");
    };
    let [Value::Int(value)] = values.as_slice() else {
        panic!("{values:?}");
    };
    let answer = value.value();
    (program, answer)
}

#[test]
fn ordinary_bakes_create_real_wrappers_and_keep_nontrailing_slots() {
    let (program, answer) = run(
        "sum::(a:int,b:int,c:int)->int{return a+b+c;} main::()->int {f::#bake_arguments sum(b=2); return f(20,20);}",
    );
    assert_eq!(answer, 42);
    assert_eq!(
        program.procedures().len(),
        3,
        "original target, main, genuine wrapper"
    );
    let wrapper = program
        .procedures()
        .iter()
        .find(|procedure| procedure.parameters.len() == 2)
        .unwrap();
    let jai_ir::Statement::CallResults { call, .. } = &wrapper.body.statements[0] else {
        panic!("wrapper contains its genuine original call");
    };
    assert_eq!(
        call.arguments
            .iter()
            .map(|(parameter, _)| parameter.index())
            .collect::<Vec<_>>(),
        [0, 1, 2]
    );
    assert_eq!(
        program
            .procedure_by_id(call.procedure)
            .unwrap()
            .parameters
            .len(),
        3
    );
}

#[test]
fn floating_bakes_preserve_the_declared_type_and_definition_scope() {
    let (_, answer) = run(
        "Bias::9; mult::(a:float,b:float)->float{return a*b;} baked::#bake_arguments mult(b=-Bias); main::()->int {Bias::100; return cast(int)baked(-4.666666666666666);}",
    );
    assert_eq!(answer, 42);
}

#[test]
fn baked_type_formals_select_the_genuine_generic_body() {
    let (program, answer) = run(
        "add::(value:T,$T:Type)->T {return value+2;} main::()->int {f::#bake_arguments add(T=int); return f(40);}",
    );
    assert_eq!(answer, 42);
    assert_eq!(
        program.procedures().len(),
        2,
        "an already-erased baked formal needs no runtime wrapper"
    );
}

#[test]
fn named_defaults_and_result_obligations_survive_wrapper_metadata() {
    let (_, answer) = run(
        "sum::(a:int,b:int,c:int=20)->int{return a+b+c;} main::()->int {callback:=#bake_arguments sum(b=2); return callback(a=20);}",
    );
    assert_eq!(answer, 42);
}

#[test]
fn duplicate_unknown_and_runtime_bakes_are_located_errors() {
    for (source, message) in [
        (
            "sum::(a:int,b:int)->int{return a+b;}main::(){f::#bake_arguments sum(b=1,b=2);}",
            "duplicate",
        ),
        (
            "sum::(a:int,b:int)->int{return a+b;}main::(){f::#bake_arguments sum(c=1);}",
            "unknown baked",
        ),
        (
            "sum::(a:int,b:int)->int{return a+b;}main::(){value:int=1;f::#bake_arguments sum(b=value);}",
            "constant",
        ),
    ] {
        let error = compile(source).unwrap_err();
        assert!(error.contains(message), "{error}");
    }
}

#[test]
fn ready_enum_and_distinct_types_reach_value_and_annotation_queries() {
    let (_, answer) = run(
        "Choice::enum{A::42;} Count::#type,distinct int; identity::(value:Count)->Count{return value;} main::()->int {callback:=identity; value:=callback(cast(Count)42); if type_of(value)==Count && type_of(Choice.A)==Choice return cast(int)value;return 0;}",
    );
    assert_eq!(answer, 42);
}
