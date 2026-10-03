//! Independent closed source fixtures check effects, cleanup and empty-loop fuel.
use jai_modules::{GraphOptions, ModuleGraph, SourceOverlay};
use jai_sema::{ResolveOptions, resolve_graph_with_options};
use jai_vm::{ByteTarget, ExecutionPhase, Limits, NoEffects, Outcome, Value, Vm};
use std::path::Path;

fn program(source: &str) -> jai_ir::Program {
    let path = Path::new("/jai-termination/main.jai");
    let mut sources = SourceOverlay::new();
    sources.insert(path, source.as_bytes().to_vec()).unwrap();
    let graph = ModuleGraph::load_with_provider(path, GraphOptions::default(), &sources).unwrap();
    let options = ResolveOptions {
        layout: Some(jai_types::LayoutPolicy::lp64()),
        ..Default::default()
    };
    resolve_graph_with_options(&graph, &options, &mut NoEffects).unwrap()
}

fn execute(source: &str, limits: Limits) -> jai_vm::Execution {
    let program = program(source);
    let entry = match program.entry() {
        jai_ir::EntryPoint::Void(id) | jai_ir::EntryPoint::Int(id) => id,
    };
    let mut vm = Vm::new_with_execution_phase(
        &program,
        NoEffects,
        limits,
        ByteTarget::default(),
        ExecutionPhase::Runtime,
    )
    .unwrap();
    vm.execute(entry, vec![])
}

fn answer(source: &str) -> i128 {
    let execution = execute(source, Limits::default());
    let Outcome::Complete(values) = execution.outcome else {
        panic!("{execution:?}")
    };
    let [Value::Int(value)] = values.as_slice() else {
        panic!("{values:?}")
    };
    value.value()
}

#[test]
fn empty_control_bodies_preserve_conditions_loops_and_enclosing_defer_cleanup() {
    assert_eq!(
        answer(
            "effects:int=0; main::()->int{;;; {defer effects+=5; if true {effects+=2;}; while false ; for i:0..2 ; if effects!=2 return 1;} return effects+35;}"
        ),
        42
    );
}

#[test]
fn empty_infinite_loop_still_consumes_bounded_fuel() {
    let execution = execute(
        "main::(){while true ;}",
        Limits {
            fuel: 32,
            ..Default::default()
        },
    );
    assert_eq!(
        execution.outcome,
        Outcome::Failed(jai_vm::Error::Limit(jai_vm::LimitKind::Fuel))
    );
    assert!(execution.statistics.steps > 0);
}

#[test]
fn quote_block_is_inert_until_real_insertion_and_keeps_its_effects() {
    assert_eq!(
        answer(
            "effects:int=0; main::()->int{quoted::#code{effects+=40;} if effects!=0 return 1; #insert quoted; return effects+2;}"
        ),
        42
    );
}

#[test]
fn here_string_assignment_keeps_every_content_byte() {
    assert_eq!(
        answer("main::()->int{text:=\"before\";text=#string END\nabc\nEND\nreturn text.count+38;}"),
        42
    );
}

#[test]
fn anonymous_callable_and_compile_time_field_initializer_execute_normally() {
    assert_eq!(
        answer("main::()->int{value:=()->int{return 40;} return value()+2;}"),
        42
    );
    assert_eq!(
        answer(
            "State::struct{value:int;} main::()->int{state:State;state.value=#run -> int{return 40;} state.value+=2;return state.value;}"
        ),
        42
    );
}

#[test]
fn empty_enum_separators_preserve_implicit_values() {
    assert_eq!(
        answer("E::enum u8{;A::7;;B;;;C;} main::()->int{return cast(int)E.B+34;}"),
        42
    );
}

#[test]
fn unsupported_anonymous_local_storage_is_diagnosed_instead_of_discarded() {
    let path = Path::new("/jai-termination/main.jai");
    let source = "main::()->int{union{a:u64;b:float64;} return 42;}";
    let mut sources = SourceOverlay::new();
    sources.insert(path, source.as_bytes().to_vec()).unwrap();
    let graph = ModuleGraph::load_with_provider(path, GraphOptions::default(), &sources).unwrap();
    let error =
        resolve_graph_with_options(&graph, &ResolveOptions::default(), &mut NoEffects).unwrap_err();
    assert!(
        error.message.contains("type values must be used"),
        "{error:?}"
    );
    let origin = graph
        .sources()
        .get(error.location.source)
        .expect("retained diagnostic source");
    assert_eq!(origin.path(), path);
    assert_eq!(origin.text(), source);
    assert_eq!(error.location.span.text(source), "union{a:u64;b:float64;}");
}
