use jai_modules::{GraphOptions, ModuleGraph, SourceOverlay};
use jai_sema::{LibraryReadiness, PreparedLibrarySession, ResolveOptions};
use std::path::Path;

fn graph(source: &str) -> ModuleGraph {
    let path = Path::new("/prepared-global-initializers/main.jai");
    let mut provider = SourceOverlay::new();
    provider.insert(path, source.as_bytes().to_vec()).unwrap();
    ModuleGraph::load_with_provider(path, GraphOptions::default(), &provider).unwrap()
}

fn program(source: &str) -> jai_ir::Program {
    let graph = graph(source);
    let mut session = PreparedLibrarySession::new(&graph, &ResolveOptions::default()).unwrap();
    let LibraryReadiness::Complete(library) = session.drive(&mut jai_vm::NoEffects) else {
        panic!("authored source must complete its genuine initializer jobs")
    };
    let main = graph
        .declarations()
        .iter()
        .find(|source| graph.symbols().name(source.name()) == "main")
        .unwrap();
    let entry = library.procedure(main.id()).unwrap().id;
    library
        .into_program(jai_ir::EntryPoint::Int(entry))
        .unwrap()
}

#[test]
fn aggregate_global_retains_anonymous_callback_body_and_source_identity() {
    let program = program(
        "Listener::struct { callback:(value:s32)->s32; } \
         listeners:[1]Listener=.[Listener.{ callback=(value:s32)->s32{return value+2;} }]; \
         main::()->int{return listeners[0].callback(40);}",
    );
    let outcome = jai_vm::execute(&program, Default::default()).outcome;
    let jai_vm::Outcome::Complete(values) = outcome else {
        panic!("{outcome:?}")
    };
    assert_eq!(values[0].integer().unwrap().value(), 42);
}

#[test]
fn retained_initializer_publication_refreshes_global_alignment_owner() {
    let program = program("buffer:[7]u8 #align 64; main::()->int{return 42;}");
    assert_eq!(
        program
            .storage_alignments()
            .global(jai_ir::GlobalId::new(0)),
        Some(64)
    );
    let outcome = jai_vm::execute(&program, Default::default()).outcome;
    let jai_vm::Outcome::Complete(values) = outcome else {
        panic!("{outcome:?}")
    };
    assert_eq!(values[0].integer().unwrap().value(), 42);
}
