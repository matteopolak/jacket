use jai_modules::{GraphDiscovery, GraphOptions, SourceOverlay};
use jai_sema::{
    DiscoveryReadiness, PreparedDiscoveryOutcome, PreparedDiscoveryRequests,
    PreparedDiscoverySession, PreparedLibrarySession, ResolveOptions,
};
use std::path::Path;

#[test]
fn original_placeholder_wait_can_admit_a_real_source_declaration() {
    let path = Path::new("/prepared-source-insertion/main.jai");
    let mut provider = SourceOverlay::new();
    provider
        .insert(
            path,
            b"#placeholder Later; Alias::#type *Later; \
            #insert #code {Later::struct {value:int;}}; \
            main::()->int{return 42;}"
                .to_vec(),
        )
        .unwrap();
    let mut discovery = GraphDiscovery::new(path, GraphOptions::default(), &provider).unwrap();
    assert!(!discovery.advance().unwrap().is_complete());
    let requests = discovery.pending_insertion_requests().cloned().collect();
    let original = discovery
        .graph()
        .declarations()
        .iter()
        .map(|source| (source.id(), source.location()))
        .collect::<Vec<_>>();
    let decision = {
        let mut session = PreparedDiscoverySession::with_insertion_admission(
            discovery.graph(),
            &ResolveOptions::default(),
            PreparedDiscoveryRequests::Insertions(requests),
            Box::new(|request, code| discovery.admit_insertion(request, code)),
        )
        .unwrap();
        let DiscoveryReadiness::Complete(PreparedDiscoveryOutcome::Insertions(mut outcome)) =
            session.drive(&mut jai_vm::NoEffects)
        else {
            panic!("original source insertion must become ready before the blocked alias")
        };
        assert_eq!(outcome.decisions.len(), 1);
        assert!(outcome.pending.is_empty());
        outcome.decisions.pop().unwrap()
    };
    assert_eq!(
        discovery
            .graph()
            .declarations()
            .iter()
            .map(|source| (source.id(), source.location()))
            .collect::<Vec<_>>(),
        original
    );
    let receipt = discovery
        .prepare_insertion_admitted(decision.request, decision.admission)
        .unwrap();
    discovery.commit_insertion(receipt).unwrap();
    assert!(discovery.advance().unwrap().is_complete());
    let graph = discovery.into_graph().ok().unwrap();
    let mut library = PreparedLibrarySession::new(&graph, &ResolveOptions::default()).unwrap();
    let jai_sema::LibraryReadiness::Complete(library) = library.drive(&mut jai_vm::NoEffects)
    else {
        panic!("the admitted declaration must satisfy the original alias")
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
        panic!("admitted source must execute")
    };
    assert_eq!(values[0].integer().unwrap().value(), 42);
}

#[test]
fn one_immutable_frontier_returns_only_its_first_admitted_insertion() {
    let path = Path::new("/prepared-source-insertion/two.jai");
    let mut provider = SourceOverlay::new();
    provider
        .insert(
            path,
            b"#insert #code {first::41;}; #insert #code {second::42;}; main::()->int{return 42;}"
                .to_vec(),
        )
        .unwrap();
    let mut discovery = GraphDiscovery::new(path, GraphOptions::default(), &provider).unwrap();
    assert!(!discovery.advance().unwrap().is_complete());
    let requests = discovery
        .pending_insertion_requests()
        .cloned()
        .collect::<Vec<_>>();
    assert_eq!(requests.len(), 2);
    let first = requests[0].id;
    let mut session = PreparedDiscoverySession::with_insertion_admission(
        discovery.graph(),
        &ResolveOptions::default(),
        PreparedDiscoveryRequests::Insertions(requests),
        Box::new(|request, code| discovery.admit_insertion(request, code)),
    )
    .unwrap();
    let outcome = session.drive(&mut jai_vm::NoEffects);
    let DiscoveryReadiness::Complete(PreparedDiscoveryOutcome::Insertions(outcome)) = outcome
    else {
        panic!("source quote must be ready")
    };
    assert_eq!(outcome.decisions.len(), 1);
    assert_eq!(outcome.decisions[0].request, first);
}
