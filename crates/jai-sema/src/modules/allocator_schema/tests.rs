use super::*;
use jai_modules::{GraphOptions, PreludeSource, SourceOverlay};
use jai_types::{Architecture, BuildTarget, ByteOrder, LayoutPolicy, OperatingSystem};
use std::path::Path;

// Independently authored declarations preserve the statically inspected source
// contract without loading a Jai compiler or native runtime.
const ALLOCATOR: &str = r#"
Allocator_Proc :: #type (mode:Allocator_Mode,size:s64,old_size:s64,old_memory:*void,allocator_data:*void)->*void;
Allocator :: struct { proc:Allocator_Proc; data:*void; }
Allocator_Mode :: enum {
    ALLOCATE::0; RESIZE::1; FREE::2; STARTUP::3; SHUTDOWN::4;
    THREAD_START::5; THREAD_STOP::6; CREATE_HEAP::7; DESTROY_HEAP::8;
    IS_THIS_YOURS::9; CAPS::10;
}
"#;

fn check(preload: &str, application: &str) -> Result<Library, String> {
    let root = Path::new("/jai-allocator-schema/main.jai");
    let mut provider = SourceOverlay::new();
    provider
        .insert(root, application.as_bytes().to_vec())
        .unwrap();
    let mut schema = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../stdlib/prelude/module.jai"
    ))
    .to_owned();
    schema.push_str(preload);
    provider
        .insert(
            Path::new("/jai-allocator-schema/modules/module.jai"),
            schema.into_bytes(),
        )
        .unwrap();
    provider
        .insert(
            Path::new("/jai-allocator-schema/modules/Foreign.jai"),
            ALLOCATOR.as_bytes().to_vec(),
        )
        .unwrap();
    let target = BuildTarget {
        operating_system: OperatingSystem::Linux,
        architecture: Architecture::X86_64,
        layout: LayoutPolicy::lp64(),
        byte_order: ByteOrder::Little,
    };
    let graph = ModuleGraph::load_with_bootstrap(
        root,
        GraphOptions {
            import_dirs: vec!["/jai-allocator-schema/modules".into()],
        },
        PreludeSource::Search,
        &provider,
        Some(target.clone()),
    )
    .map_err(|error| format!("{error:?}"))?;
    crate::resolve_library_with_options(
        &graph,
        &crate::ResolveOptions {
            target: Some(target),
            ..Default::default()
        },
        &mut jai_vm::NoEffects,
    )
    .map_err(|error| error.render(graph.sources()))
}

#[test]
fn selected_preload_role_survives_an_application_lookalike() {
    let library = check(
        ALLOCATOR,
        "Allocator :: struct { unrelated:int; } shadow:Allocator;",
    )
    .unwrap();
    let schema = library.types().allocator_schema().expect("authentic role");
    let shadow = library.globals().last().unwrap();
    assert_ne!(shadow.place().ty(), schema.ty());
    assert_eq!(
        library
            .types()
            .record_definition(schema.ty())
            .unwrap()
            .fields
            .len(),
        2
    );
}

#[test]
fn missing_selected_role_does_not_adopt_application_types() {
    let library = check("", ALLOCATOR).unwrap();
    assert!(library.types().allocator_schema().is_none());
}

#[test]
fn incompatible_selected_source_contract_is_rejected() {
    for source in [
        ALLOCATOR.replace("proc:Allocator_Proc", "function:Allocator_Proc"),
        ALLOCATOR.replace("CAPS::10", "CAPABILITIES::10"),
        ALLOCATOR.replace("old_size:s64", "old_size:u64"),
        ALLOCATOR.replace("->*void;", "->*void #no_context;"),
    ] {
        let error = check(&source, "").unwrap_err();
        assert!(error.contains("selected Preload"), "{error}");
    }
}

#[test]
fn a_matching_foreign_reexport_cannot_designate_the_preload_role() {
    let error = check("#import \"Foreign\";", "").unwrap_err();
    assert!(error.contains("not a reexport"), "{error}");
}
