//! Single-item branches preserve real dependency selection without body rewrites.
use jai_modules::{GraphOptions, ModuleGraph, SourceOverlay};
use std::path::{Path, PathBuf};

fn graph(main: &str, selected: (&str, &str)) -> ModuleGraph {
    let mut source = SourceOverlay::new();
    source
        .insert(Path::new("/file-if/main.jai"), main.as_bytes().to_vec())
        .unwrap();
    source
        .insert(Path::new(selected.0), selected.1.as_bytes().to_vec())
        .unwrap();
    ModuleGraph::load_with_provider(
        Path::new("/file-if/main.jai"),
        GraphOptions {
            import_dirs: vec![PathBuf::from("/file-if")],
        },
        &source,
    )
    .unwrap()
}

#[test]
fn unbraced_load_selects_only_the_chosen_source_and_keeps_the_next_declaration() {
    let graph = graph(
        "#if true #load \"selected.jai\"; else #load \"missing.jai\"; FOLLOW::42;",
        ("/file-if/selected.jai", "SELECTED::7;"),
    );
    assert_eq!(graph.sources().records().len(), 2);
    let exports = graph.module(graph.root()).unwrap().exports();
    assert!(exports.contains_key(&graph.symbols().find("SELECTED").unwrap()));
    assert!(exports.contains_key(&graph.symbols().find("FOLLOW").unwrap()));
}

#[test]
fn an_unbraced_else_if_import_has_its_normal_module_identity() {
    let graph = graph(
        "#if false #load \"missing.jai\"; else #if true #import \"Selected\"; else #import \"Missing\"; FOLLOW::42;",
        ("/file-if/Selected/module.jai", "SELECTED::7;"),
    );
    assert_eq!(graph.sources().records().len(), 2);
    assert_eq!(graph.modules().len(), 2);
    assert_eq!(graph.imports().len(), 1);
    let exports = graph.module(graph.root()).unwrap().exports();
    assert!(exports.contains_key(&graph.symbols().find("SELECTED").unwrap()));
    assert!(exports.contains_key(&graph.symbols().find("FOLLOW").unwrap()));
}
