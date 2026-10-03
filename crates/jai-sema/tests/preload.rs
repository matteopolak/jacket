use jai_modules::{GraphOptions, ModuleGraph, PreludeSource, SourceOverlay};
use jai_types::{Architecture, BuildTarget, ByteOrder, LayoutPolicy, OperatingSystem};
use std::path::Path;

#[test]
fn complete_compiler_prelude_resolves_with_its_source_schema_and_intrinsics() {
    let source = jai_modules::compiler_prelude_source();
    let mut provider = SourceOverlay::new();
    provider
        .insert(
            Path::new("/jai-sema-preload/main.jai"),
            b"main :: () {}".to_vec(),
        )
        .unwrap();
    provider
        .insert(
            Path::new("/jai-sema-preload/modules/module.jai"),
            source.as_bytes().to_vec(),
        )
        .unwrap();
    // An explicitly selected target makes layout-dependent intrinsic ABI checks
    // reproducible; this is not a host target inference or native execution.
    let target = BuildTarget {
        operating_system: OperatingSystem::Linux,
        architecture: Architecture::X86_64,
        layout: LayoutPolicy::lp64(),
        byte_order: ByteOrder::Little,
    };
    let roots = vec!["/jai-sema-preload/modules".into()];
    let graph = ModuleGraph::load_with_bootstrap(
        Path::new("/jai-sema-preload/main.jai"),
        GraphOptions {
            import_dirs: roots.clone(),
        },
        PreludeSource::Search,
        &provider,
        Some(target.clone()),
    )
    .unwrap();
    let workspace = jai_vm::WorkspaceId::from_raw(1).unwrap();
    let compiler = jai_sema::CompilerBindingContext::from_graph(&graph, &roots, workspace);
    let options = jai_sema::ResolveOptions {
        target: Some(target),
        compiler: Some(compiler),
        ..Default::default()
    };
    let library = jai_sema::resolve_library_with_options(&graph, &options, &mut jai_vm::NoEffects)
        .unwrap_or_else(|error| panic!("{}", error.render(graph.sources())));
    assert_eq!(
        graph
            .sources()
            .get(
                graph
                    .file(graph.module(graph.prelude().unwrap()).unwrap().entry())
                    .unwrap()
                    .source()
            )
            .unwrap()
            .text(),
        source
    );
    assert!(
        library
            .prototypes()
            .iter()
            .any(|prototype| matches!(prototype.origin, jai_sema::PrototypeOrigin::Intrinsic(_)))
    );
}
