use jai_modules::{GraphOptions, ModuleGraph, SourceOverlay};
use jai_sema::{LibraryReadiness, PreparedLibrarySession, ResolveOptions, SourcePrefixReadiness};
use std::path::Path;

fn graph(source: &str) -> ModuleGraph {
    let path = Path::new("/prepared-source-prefix/main.jai");
    let mut provider = SourceOverlay::new();
    provider.insert(path, source.as_bytes().to_vec()).unwrap();
    ModuleGraph::load_with_provider(path, GraphOptions::default(), &provider).unwrap()
}

#[test]
fn prefix_stops_after_each_original_run_before_unrelated_body_failure() {
    let graph = graph(
        "first::()->int{return 1;} #run first(); \
         second::()->int{return 2;} #run second(); \
         unrelated::()->int{return missing;} main::()->int{return 42;}",
    );
    let mut session = PreparedLibrarySession::new(&graph, &ResolveOptions::default()).unwrap();
    assert!(matches!(
        session.drive_source_prefix(&mut jai_vm::NoEffects),
        SourcePrefixReadiness::Ready
    ));
    assert!(matches!(
        session.drive_source_prefix(&mut jai_vm::NoEffects),
        SourcePrefixReadiness::Ready
    ));
    assert!(matches!(
        session.drive_source_prefix(&mut jai_vm::NoEffects),
        SourcePrefixReadiness::Complete
    ));
    let LibraryReadiness::Failed(error) = session.drive(&mut jai_vm::NoEffects) else {
        panic!("the original unrelated body remains a genuine terminal error")
    };
    assert!(error.message.contains("missing"));
}

#[test]
fn original_empty_context_allows_source_run_before_a_real_placeholder_wait() {
    let graph = graph(
        "#placeholder Later; Alias::#type *Later; \
         first::()->int{return 1;} #run first(); main::()->int{return 42;}",
    );
    let marker = graph.placeholders()[0].id();
    let mut session = PreparedLibrarySession::new(&graph, &ResolveOptions::default()).unwrap();
    assert!(matches!(
        session.drive_source_prefix(&mut jai_vm::NoEffects),
        SourcePrefixReadiness::Ready
    ));
    assert!(matches!(
        session.drive_source_prefix(&mut jai_vm::NoEffects),
        SourcePrefixReadiness::Complete
    ));
    let LibraryReadiness::Pending(pending) = session.drive(&mut jai_vm::NoEffects) else {
        panic!("prefix completion cannot fill an unfulfilled source placeholder")
    };
    assert!(pending.dependencies.is_empty());
    assert_eq!(pending.source.unwrap().placeholder(), Some(marker));
    session.cancel(&mut jai_vm::NoEffects).unwrap();
}

#[test]
fn prefix_completes_selected_initial_scalar_jobs_before_full_binding() {
    let graph = graph(
        "count::()->int #no_context{return 42;} N:int:#run count(); \
         Alias::#type [N]u8; global:Alias; main::()->int{return global.count;}",
    );
    let mut session = PreparedLibrarySession::new(&graph, &ResolveOptions::default()).unwrap();
    assert!(matches!(
        session.drive_source_prefix(&mut jai_vm::NoEffects),
        SourcePrefixReadiness::Complete
    ));
    let LibraryReadiness::Complete(library) = session.drive(&mut jai_vm::NoEffects) else {
        panic!("the selected scalar producer must retain its checked result")
    };
    let main = graph
        .declarations()
        .iter()
        .find(|source| graph.symbols().name(source.name()) == "main")
        .unwrap();
    let entry = library.procedure(main.id()).unwrap().id;
    let program = library
        .into_program(jai_ir::EntryPoint::Int(entry))
        .unwrap();
    let jai_vm::Outcome::Complete(values) = jai_vm::execute(&program, Default::default()).outcome
    else {
        panic!("actual prepared source must execute")
    };
    assert_eq!(values[0].integer().unwrap().value(), 42);
}

#[test]
fn prefix_requests_actual_alignment_body_before_global_allocation() {
    let graph = graph(
        "alignment::()->int #no_context{return 64;} \
         buffer:[7]u8 #align #run alignment(); main::()->int{return 42;}",
    );
    let mut session = PreparedLibrarySession::new(&graph, &ResolveOptions::default()).unwrap();
    assert!(matches!(
        session.drive_source_prefix(&mut jai_vm::NoEffects),
        SourcePrefixReadiness::Complete
    ));
    let LibraryReadiness::Complete(library) = session.drive(&mut jai_vm::NoEffects) else {
        panic!("alignment prerequisites must use the actual retained body")
    };
    let main = graph
        .declarations()
        .iter()
        .find(|source| graph.symbols().name(source.name()) == "main")
        .unwrap();
    let entry = library.procedure(main.id()).unwrap().id;
    let program = library
        .into_program(jai_ir::EntryPoint::Int(entry))
        .unwrap();
    assert_eq!(
        program
            .storage_alignments()
            .global(jai_ir::GlobalId::new(0)),
        Some(64)
    );
}

#[test]
fn original_prefix_waits_for_real_procedure_defaults() {
    let graph = graph(
        "read::(to_standard_error:=false)->int #no_context {if to_standard_error return 0;return 42;} \
         #run read(); main::()->int{return read();}",
    );
    let mut session = PreparedLibrarySession::new(&graph, &ResolveOptions::default()).unwrap();
    assert!(matches!(
        session.drive_source_prefix(&mut jai_vm::NoEffects),
        SourcePrefixReadiness::Ready
    ));
    assert!(matches!(
        session.drive_source_prefix(&mut jai_vm::NoEffects),
        SourcePrefixReadiness::Complete
    ));
    let library = match session.drive(&mut jai_vm::NoEffects) {
        LibraryReadiness::Complete(library) => *library,
        LibraryReadiness::Failed(error) => panic!("{error:?}"),
        LibraryReadiness::Pending(pending) => panic!("{pending:?}"),
    };
    let main = graph
        .declarations()
        .iter()
        .find(|source| graph.symbols().name(source.name()) == "main")
        .unwrap();
    let entry = library.procedure(main.id()).unwrap().id;
    let program = library
        .into_program(jai_ir::EntryPoint::Int(entry))
        .unwrap();
    let jai_vm::Outcome::Complete(values) = jai_vm::execute(&program, Default::default()).outcome
    else {
        panic!("the real declared default must complete both source and runtime calls")
    };
    assert_eq!(values[0].integer().unwrap().value(), 42);
}
