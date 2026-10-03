use jai_modules::{GraphOptions, GraphSyntaxStorage, ModuleGraph, SourceOverlay};
use std::path::Path;

fn graph() -> ModuleGraph {
    let mut sources = SourceOverlay::new();
    sources
        .insert(
            Path::new("/storage/main.jai"),
            b"ANSWER :: 42; main :: () {}".to_vec(),
        )
        .unwrap();
    ModuleGraph::load_with_provider(
        Path::new("/storage/main.jai"),
        GraphOptions::default(),
        &sources,
    )
    .unwrap()
}

#[test]
fn census_lends_both_parser_and_independently_registered_declarations() {
    let graph = graph();
    let mut parsed = 0;
    let mut declarations = 0;
    let mut sources = 0;
    let mut bytes = 0;
    graph
        .visit_retained_source_storage(
            &mut |_| Ok::<_, ()>(()),
            &mut |n| {
                bytes += n;
                Ok(())
            },
            &mut |_| {
                sources += 1;
                Ok(())
            },
            &mut |root| {
                match root {
                    GraphSyntaxStorage::ParsedFile(_) => parsed += 1,
                    GraphSyntaxStorage::FileDeclaration(_) => declarations += 1,
                    _ => {}
                };
                Ok(())
            },
            &mut |_| Ok(()),
            &mut |_| Ok(()),
        )
        .unwrap();
    assert_eq!(parsed, graph.files().len());
    assert_eq!(declarations, graph.declarations().len());
    assert_eq!(sources, graph.sources().records().len());
    assert!(bytes > std::mem::size_of::<ModuleGraph>());
}

#[test]
fn first_work_denial_precedes_all_owner_projections() {
    let graph = graph();
    let mut projected = 0;
    let result = graph.visit_retained_source_storage(
        &mut |_| Err("stop"),
        &mut |_| Ok(()),
        &mut |_| {
            projected += 1;
            Ok(())
        },
        &mut |_| Ok(()),
        &mut |_| Ok(()),
        &mut |_| Ok(()),
    );
    assert_eq!(result, Err("stop"));
    assert_eq!(projected, 0);
}
