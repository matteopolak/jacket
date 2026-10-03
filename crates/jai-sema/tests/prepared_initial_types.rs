use jai_modules::{GraphOptions, ModuleGraph, SourceOverlay};
use jai_sema::{LibraryReadiness, PreparedLibrarySession, ResolveOptions};
use std::path::Path;

fn graph(source: &str) -> ModuleGraph {
    let path = Path::new("/prepared-initial-types/main.jai");
    let mut provider = SourceOverlay::new();
    provider.insert(path, source.as_bytes().to_vec()).unwrap();
    ModuleGraph::load_with_provider(path, GraphOptions::default(), &provider).unwrap()
}

fn run(source: &str) -> i128 {
    let graph = graph(source);
    let mut session = PreparedLibrarySession::new(&graph, &ResolveOptions::default()).unwrap();
    let library = match session.drive(&mut jai_vm::NoEffects) {
        LibraryReadiness::Complete(library) => library,
        LibraryReadiness::Pending(pending) => panic!("{pending:?}"),
        LibraryReadiness::Failed(error) => panic!("{error:?}"),
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
    let outcome = jai_vm::execute(&program, Default::default()).outcome;
    let jai_vm::Outcome::Complete(values) = outcome else {
        panic!("{outcome:?}")
    };
    values[0].integer().unwrap().value()
}

#[test]
fn count_only_modifier_runs_with_actual_no_context_before_alias_publication() {
    assert_eq!(
        run(
            "Buffer::struct(N:int=3) #modify {N+=1;return true;} {values:[N]int;} \
        Alias::#type Buffer(); global:Alias; main::()->int{return global.values.count;}"
        ),
        4
    );
}

#[test]
fn generated_count_constant_finishes_its_real_prerequisite_body() {
    assert_eq!(
        run(
            "count::()->int #no_context{return 42;} N:int:#run count(); \
        Alias::#type [N]u8; global:Alias; main::()->int{return global.count;}"
        ),
        42
    );
}

#[test]
fn global_and_header_annotations_keep_modifier_requests_until_canonical_ready() {
    assert_eq!(
        run(
            "Buffer::struct(N:int=3) #modify {N+=1;return true;} {values:[N]int;} \
        global:Buffer(); read::(value:Buffer())->int{return value.values.count;} \
        main::()->int{return read(global);}"
        ),
        4
    );
}

#[test]
fn fixed_unfilled_placeholder_keeps_source_identity_without_a_vm_dependency() {
    let graph = graph("#placeholder Later; Alias::#type *Later; main::()->int{return 42;}");
    let marker = graph.placeholders()[0].id();
    let alias = &graph.declarations()[0];
    let mut session = PreparedLibrarySession::new(&graph, &ResolveOptions::default()).unwrap();
    for _ in 0..2 {
        let LibraryReadiness::Pending(pending) = session.drive(&mut jai_vm::NoEffects) else {
            panic!("a source marker must remain a source wait")
        };
        assert!(pending.dependencies.is_empty());
        let source = pending.source.expect("actual source readiness cause");
        assert_eq!(source.placeholder(), Some(marker));
        assert_eq!(source.location().source, alias.location().source);
        assert_ne!(source.location(), graph.placeholders()[0].location());
        assert!(
            pending
                .diagnostic
                .message
                .contains("placeholder reserved here")
        );
    }
    session.cancel(&mut jai_vm::NoEffects).unwrap();
    assert!(matches!(
        session.drive(&mut jai_vm::NoEffects),
        LibraryReadiness::Failed(_)
    ));
}
