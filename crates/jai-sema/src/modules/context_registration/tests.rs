use super::*;
use jai_modules::{
    BootstrapOptions, GraphOptions, PreludeSource, RuntimeSupportOptions, RuntimeSupportParameters,
    RuntimeSupportSource, SourceOverlay,
};
use std::path::Path;

fn graph(activate_runtime: bool) -> ModuleGraph {
    let mut provider = SourceOverlay::new();
    for (path, source) in [
        (
            "/jai-first-context/main.jai",
            "FIRST_ADD_CONTEXT :: #code #add_context application_field: int; main :: () {}",
        ),
        (
            "/jai-first-context/modules/module.jai",
            "preload_value :: 1;",
        ),
        (
            "/jai-first-context/modules/Runtime_Support.jai",
            "#module_parameters(DEFINE_SYSTEM_ENTRY_POINT: bool, DEFINE_INITIALIZATION: bool, ENABLE_BACKTRACE_ON_CRASH: bool);",
        ),
    ] {
        provider
            .insert(Path::new(path), source.as_bytes().to_vec())
            .unwrap();
    }
    ModuleGraph::load_with_bootstrap_options(
        Path::new("/jai-first-context/main.jai"),
        GraphOptions {
            import_dirs: vec!["/jai-first-context/modules".into()],
        },
        BootstrapOptions {
            prelude: PreludeSource::Search,
            runtime_support: activate_runtime.then_some(RuntimeSupportOptions {
                source: RuntimeSupportSource::Search,
                parameters: RuntimeSupportParameters {
                    define_system_entry_point: false,
                    define_initialization: true,
                    enable_backtrace_on_crash: false,
                    temporary_storage_size: 32768,
                },
            }),
        },
        &provider,
        None,
    )
    .unwrap()
}

#[test]
fn activated_runtime_requires_its_designated_preload_marker_not_application_shadow() {
    let graph = graph(true);
    let error = collect(&graph, &mut Constants::new(&graph))
        .err()
        .expect("missing canonical marker");
    assert!(
        error
            .message
            .contains("Preload must export FIRST_ADD_CONTEXT")
    );
    assert_eq!(
        graph.sources().get(error.location.source).unwrap().path(),
        Path::new("/jai-first-context/modules/module.jai")
    );
}

#[test]
fn source_only_preload_without_runtime_does_not_activate_quoted_context_fields() {
    let graph = graph(false);
    let registration = collect(&graph, &mut Constants::new(&graph)).unwrap();
    assert!(registration.fields.is_empty());
    assert!(registration.consumed.is_empty());
}
