//! Inferred header domains follow actual enum alias definitions without reading values.
use jai_modules::{GraphOptions, ModuleGraph, SourceOverlay};
use jai_sema::{LibraryReadiness, PreparedLibrarySession, ResolveOptions};
use std::path::Path;
fn graph(source: &str) -> ModuleGraph {
    let mut overlay = SourceOverlay::new();
    overlay
        .insert(
            Path::new("/enum-alias-defaults/main.jai"),
            source.as_bytes().to_vec(),
        )
        .unwrap();
    overlay
        .insert(
            Path::new("/enum-alias-defaults/modules/Protocol/module.jai"),
            b"Flags::enum_flags u32{ERROR::2;TRACE::4;} Other::enum_flags u32{ERROR::2;}".to_vec(),
        )
        .unwrap();
    ModuleGraph::load_with_provider(
        Path::new("/enum-alias-defaults/main.jai"),
        GraphOptions {
            import_dirs: vec!["/enum-alias-defaults/modules".into()],
        },
        &overlay,
    )
    .unwrap()
}
#[test]
fn inferred_default_alias_retains_the_imported_canonical_enum() {
    let graph = graph(
        "Protocol::#import \"Protocol\"; Alias::Protocol.Flags; Further::Alias; read::(flags:=Further.ERROR)->Protocol.Flags #no_context{return flags;} main::()->int{flags:Protocol.Flags=read();return cast(int)flags;}",
    );
    let mut session = PreparedLibrarySession::new(&graph, &ResolveOptions::default()).unwrap();
    let library = match session.drive(&mut jai_vm::NoEffects) {
        LibraryReadiness::Complete(library) => library,
        LibraryReadiness::Failed(error) => panic!("{error:?}"),
        LibraryReadiness::Pending(wait) => panic!("{wait:?}"),
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
    assert_eq!(values[0].integer().unwrap().value(), 2);
}
#[test]
fn unknown_alias_enum_member_is_a_real_source_error() {
    let graph = graph(
        "Protocol::#import \"Protocol\"; Alias::Protocol.Flags; read::(flags:=Alias.MISSING)->int #no_context{return 42;} main::()->int{return read();}",
    );
    let error = PreparedLibrarySession::new(&graph, &ResolveOptions::default())
        .err()
        .expect("unknown original enum member");
    assert!(error.message.contains("unknown enum member"), "{error:?}");
    assert!(!error.message.contains("not a namespace"));
}
