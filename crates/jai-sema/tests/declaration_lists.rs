//! Real checked declaration groups preserve result arity and single evaluation.
use jai_modules::{GraphOptions, ModuleGraph, SourceOverlay};
use jai_vm::{Limits, Outcome, Value};
use std::path::Path;

fn graph(source: &str) -> ModuleGraph {
    let path = Path::new("/declaration-lists/main.jai");
    let mut overlay = SourceOverlay::new();
    overlay.insert(path, source.as_bytes().to_vec()).unwrap();
    ModuleGraph::load_with_provider(path, GraphOptions::default(), &overlay).unwrap()
}
fn checked(source: &str) -> Result<jai_ir::Program, String> {
    let graph = graph(source);
    jai_sema::resolve_graph_with_options(
        &graph,
        &jai_sema::ResolveOptions {
            layout: Some(jai_types::LayoutPolicy::lp64()),
            ..Default::default()
        },
        &mut jai_vm::NoEffects,
    )
    .map_err(|error| error.render(graph.sources()))
}
fn run(source: &str) -> i128 {
    let program = checked(source).unwrap();
    let outcome = jai_vm::execute(&program, Limits::default());
    let Outcome::Complete(values) = outcome.outcome else {
        panic!("{outcome:?}")
    };
    let [Value::Int(value)] = values.as_slice() else {
        panic!("{values:?}")
    };
    value.value()
}

#[test]
fn file_group_bindings_publish_distinct_actual_declaration_ids() {
    let graph = graph("first,second:int=21; main::()->int{return first+second;}");
    let globals: Vec<_> = graph
        .declarations()
        .iter()
        .filter(|source| {
            matches!(
                source.syntax().kind,
                jai_syntax::FileDeclarationKind::Global(_)
            )
        })
        .collect();
    assert_eq!(globals.len(), 2);
    assert_ne!(globals[0].id(), globals[1].id());
    assert_ne!(globals[0].name(), globals[1].name());
    assert_ne!(
        globals[0].location().span.start,
        globals[1].location().span.start
    );
    assert_eq!(
        run("first,second:int=21; main::()->int{return first+second;}"),
        42
    );
}

#[test]
fn local_scalar_replication_evaluates_a_nested_side_effect_once() {
    assert_eq!(
        run(
            "counter:int; tick::()->int{counter+=1;return counter;} main::()->int{a,b,c:int=tick()+20;return a+b+c+counter;}"
        ),
        64
    );
}

#[test]
fn inferred_scalar_replication_and_compound_assignment_keep_original_places() {
    assert_eq!(run("main::()->int{a,b:=20; a,b+=1; return a+b;}"), 42);
}

#[test]
fn local_multi_result_calls_execute_once_and_preserve_result_order() {
    assert_eq!(
        run(
            "counter:int; pair::()->int,int{counter+=1;return 20,21;} main::()->int{a,b:=pair();return a+b+counter;}"
        ),
        42
    );
}

#[test]
fn one_result_calls_are_not_reinterpreted_as_scalar_replication() {
    let error =
        checked("one::()->int{return 21;} main::()->int{a,b:=one();return a+b;}").unwrap_err();
    assert!(error.contains("result count"), "{error}");
}

#[test]
fn typed_uninitialized_names_are_real_storage_without_default_writes() {
    assert_eq!(
        run(
            "Point::struct{x,y:int;} main::()->int{a,b:int=---;p,q:Point=---;a,b=20,22;p.x=a;q.y=b;return p.x+q.y;}"
        ),
        42
    );
}

#[test]
fn grouped_record_defaults_and_typed_field_names_keep_their_layout() {
    assert_eq!(
        run("Point::struct{x,y:int=21;} main::()->int{p:Point;return p.x+p.y;}"),
        42
    );
}

#[test]
fn file_initializer_replication_uses_one_published_checked_value() {
    assert_eq!(
        run(
            "counter:int; tick::()->int{counter+=1;return counter;} first,second:int=tick()+20; main::()->int{return first+second;}"
        ),
        42
    );
}

#[test]
fn file_individual_initializers_keep_distinct_types_and_values() {
    assert_eq!(run("a,b:=20,22; main::()->int{return a+b;}"), 42);
}

#[test]
fn declaration_lists_reject_invalid_duplicates_and_untyped_no_write() {
    for source in [
        "main::()->int{a,a:int=21;return a;}",
        "main::()->int{a,b:=---;return 0;}",
    ] {
        assert!(checked(source).is_err(), "{source}");
    }
}

#[test]
fn file_call_result_lists_keep_a_precise_joint_publication_boundary() {
    let error = checked("pair::()->int,int{return 20,22;} a,b:=pair(); main::()->int{return a+b;}")
        .unwrap_err();
    assert!(error.contains("joint global result publication"), "{error}");
}
