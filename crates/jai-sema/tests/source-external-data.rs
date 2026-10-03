//! Independently authored declarations exercise genuine source and storage owners.
use jai_ir::{ExternalDataId, ExternalDataSource, GlobalInitializer};
use jai_modules::{GraphOptions, ModuleGraph, SourceOverlay};
use jai_sema::{ResolveOptions, resolve_graph_with_options};
use jai_vm::{Error, Limits, NoEffects, Outcome, Value};
use std::path::Path;

fn graph(source: &str) -> ModuleGraph {
    let path = Path::new("/source-external-data/main.jai");
    let mut overlay = SourceOverlay::new();
    overlay.insert(path, source.as_bytes().to_vec()).unwrap();
    ModuleGraph::load_with_provider(path, GraphOptions::default(), &overlay).unwrap()
}
fn program(source: &str) -> jai_ir::Program {
    resolve_graph_with_options(&graph(source), &ResolveOptions::default(), &mut NoEffects).unwrap()
}
fn assert_answer(program: &jai_ir::Program) {
    let execution = jai_vm::execute(program, Limits::default());
    assert!(
        matches!(execution.outcome, Outcome::Complete(ref values)
        if matches!(values.as_slice(), [Value::Int(value)] if value.value() == 42)),
        "{execution:?}"
    );
}

#[test]
fn file_external_has_actual_graph_identity_and_no_initial_value() {
    let graph = graph("counter:int #elsewhere; main::()->int{return 42;}");
    let declaration = graph
        .declarations()
        .iter()
        .find(|declaration| graph.symbols().name(declaration.name()) == "counter")
        .unwrap();
    let program =
        resolve_graph_with_options(&graph, &ResolveOptions::default(), &mut NoEffects).unwrap();
    let GlobalInitializer::External(data) = program.globals()[0].initializer() else {
        panic!("external data has no initializer")
    };
    assert_eq!(data.id(), ExternalDataId::File(declaration.id()));
    assert_eq!(data.location(), declaration.location());
    assert_eq!(data.source(), &ExternalDataSource::Program);
    assert_eq!(data.symbol(), "counter");
    assert_answer(&program);
}

#[test]
fn lexical_external_slots_follow_the_real_file_prefix_and_keep_distinct_identities() {
    let program = program(
        "base:bool=true; main::()->int{value:int #elsewhere; {value:int #elsewhere;} return 42;}",
    );
    assert!(matches!(
        program.globals()[0].initializer(),
        GlobalInitializer::Bool(true)
    ));
    let data: Vec<_> = program
        .globals()
        .iter()
        .skip(1)
        .map(|global| {
            let GlobalInitializer::External(data) = global.initializer() else {
                panic!("external storage")
            };
            data
        })
        .collect();
    assert_eq!(data.len(), 2);
    assert_eq!(data[0].symbol(), data[1].symbol());
    assert_ne!(data[0].id(), data[1].id());
    for data in data {
        let ExternalDataId::Local { procedure, .. } = data.id() else {
            panic!("actual local owner")
        };
        assert!(program.procedure_by_id(procedure).is_some());
    }
    assert_answer(&program);
}

#[test]
fn unused_anonymous_external_retains_a_checked_owner_without_a_runtime_body() {
    let program =
        program("base:bool=true; #run { value:int #elsewhere; } main::()->int{return 42;}");
    let GlobalInitializer::External(data) = program.globals()[1].initializer() else {
        panic!("external storage")
    };
    let ExternalDataId::Local { procedure, .. } = data.id() else {
        panic!("actual temporary owner")
    };
    let owner = program
        .library()
        .source_procedure_owners()
        .get(procedure)
        .expect("checked source-only receipt");
    assert!(
        owner
            .identity()
            .body_text()
            .contains("value:int #elsewhere")
    );
    assert!(program.procedure_by_id(procedure).is_none());
    assert!(program.library().signature(procedure).is_none());
    assert_answer(&program);
}

#[test]
fn real_vm_access_is_unavailable_without_a_native_storage_provider() {
    let program = program("counter:int #elsewhere; main::()->int{return counter;}");
    let execution = jai_vm::execute(&program, Limits::default());
    assert!(
        matches!(execution.outcome, Outcome::Failed(Error::UnsupportedExternalGlobal(id)) if id == program.globals()[0].id()),
        "{execution:?}"
    );
}

#[test]
fn aggregate_external_uses_its_defined_canonical_record_and_remains_non_demanding() {
    let program = program(
        "State::struct{count:int;valid:bool;} state:State #elsewhere; main::()->int{return 42;}",
    );
    let GlobalInitializer::External(data) = program.globals()[0].initializer() else {
        panic!("external record")
    };
    assert_eq!(
        program
            .types()
            .record_definition(data.ty())
            .unwrap()
            .fields
            .len(),
        2
    );
    assert_answer(&program);
}

#[test]
fn anonymous_local_library_has_a_real_certified_source_owner() {
    let program = program(
        "#run { system::#foreign_system_library \"c\"; unused:int #elsewhere system \"author_native_data\"; } main::()->int{return 42;}",
    );
    let GlobalInitializer::External(data) = program.globals()[0].initializer() else {
        panic!("external storage")
    };
    let ExternalDataSource::Library(library) = data.source() else {
        panic!("lexical library")
    };
    assert!(
        program
            .library()
            .foreign_libraries()
            .iter()
            .any(|canonical| canonical == library)
    );
    let jai_ir::ForeignLibraryId::Local { procedure, .. } = library.id else {
        panic!("local library owner")
    };
    assert!(
        program
            .library()
            .source_procedure_owners()
            .get(procedure)
            .is_some()
    );
    assert_eq!(data.symbol(), "author_native_data");
    assert_answer(&program);
}

#[test]
fn external_context_member_is_rejected_without_an_owned_initializer() {
    let graph = graph("#add_context value:int #elsewhere; main::()->int{return 42;}");
    let error =
        resolve_graph_with_options(&graph, &ResolveOptions::default(), &mut NoEffects).unwrap_err();
    assert!(
        error
            .message
            .contains("external data cannot supply an owned context field"),
        "{error:?}"
    );
    assert_eq!(
        graph.sources().get(error.location.source).unwrap().path(),
        Path::new("/source-external-data/main.jai")
    );
}
