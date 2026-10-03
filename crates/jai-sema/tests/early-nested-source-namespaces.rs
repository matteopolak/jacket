//! Canonical early nested types belong to their original source namespace.
use jai_modules::{GraphOptions, ModuleGraph, SourceOverlay};
use jai_sema::{LibraryReadiness, PreparedLibrarySession, ResolveOptions};
use std::path::Path;

fn graph(source: &str) -> ModuleGraph {
    let mut overlay = SourceOverlay::new();
    overlay
        .insert(
            Path::new("/early-nested/main.jai"),
            source.as_bytes().to_vec(),
        )
        .unwrap();
    overlay
        .insert(
            Path::new("/early-nested/modules/Protocol/module.jai"),
            b"Storage::struct{pages:*Page; Page::struct{next:*Page;size:s64;}}".to_vec(),
        )
        .unwrap();
    ModuleGraph::load_with_provider(
        Path::new("/early-nested/main.jai"),
        GraphOptions {
            import_dirs: vec!["/early-nested/modules".into()],
        },
        &overlay,
    )
    .unwrap()
}

#[test]
fn early_transparent_import_aliases_use_the_original_nested_nominal() {
    let graph = graph(
        "Protocol::#import \"Protocol\"; Alias::Protocol.Storage; Further::Alias; read::(page:*Further.Page)->int #no_context{if page return 0;return 42;} canonical::(page:*Protocol.Storage.Page)->int #no_context{return 42;} main::()->int{return read(null);}",
    );
    let mut session = PreparedLibrarySession::new(&graph, &ResolveOptions::default()).unwrap();
    let library = match session.drive(&mut jai_vm::NoEffects) {
        LibraryReadiness::Complete(library) => library,
        LibraryReadiness::Failed(error) => panic!("{error:?}"),
        LibraryReadiness::Pending(wait) => panic!("{wait:?}"),
    };
    let declaration = |name: &str| {
        graph
            .declarations()
            .iter()
            .find(|source| graph.symbols().name(source.name()) == name)
            .unwrap()
            .id()
    };
    assert_eq!(
        library.procedure(declaration("read")).unwrap().parameters[0].ty(),
        library
            .procedure(declaration("canonical"))
            .unwrap()
            .parameters[0]
            .ty()
    );
    let main = library.procedure(declaration("main")).unwrap().id;
    let program = library.into_program(jai_ir::EntryPoint::Int(main)).unwrap();
    let outcome = jai_vm::execute(&program, Default::default()).outcome;
    assert!(matches!(outcome, jai_vm::Outcome::Complete(values)
        if matches!(values.as_slice(), [jai_vm::Value::Int(value)] if value.value()==42)));
}

#[test]
fn unavailable_alias_root_retains_the_actual_placeholder_cause() {
    let graph = graph(
        "#placeholder Future; Alias::Future; read::(page:*Alias.Page)->int #no_context{return 42;} main::()->int{return 42;}",
    );
    let mut session = PreparedLibrarySession::new(&graph, &ResolveOptions::default()).unwrap();
    let LibraryReadiness::Pending(wait) = session.drive(&mut jai_vm::NoEffects) else {
        panic!("the missing original namespace owner must remain a typed source prerequisite")
    };
    assert!(wait.dependencies.is_empty());
    assert_eq!(
        wait.source.unwrap().placeholder(),
        Some(graph.placeholders()[0].id())
    );
    session.cancel(&mut jai_vm::NoEffects).unwrap();
}

#[test]
fn source_storage_shadow_does_not_acquire_a_type_namespace() {
    let graph = graph(
        "Protocol::#import \"Protocol\"; Alias: int; read::(page:*Alias.Page)->int #no_context{return 42;} main::()->int{return 42;}",
    );
    let error = PreparedLibrarySession::new(&graph, &ResolveOptions::default())
        .err()
        .unwrap();
    assert!(error.message.contains("not a namespace"), "{error:?}");
}
