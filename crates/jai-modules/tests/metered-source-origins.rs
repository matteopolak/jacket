use jai_modules::{GraphOptions, ModuleGraph, SourceOriginAdmissionError, SourceOverlay};
use std::path::Path;

fn graph() -> ModuleGraph {
    let mut sources = SourceOverlay::new();
    for (path, source) in [
        (
            "/meter/main.jai",
            "A :: #import,file \"library.jai\"(X=1,Y=2); B :: #import,file \"library.jai\"(Y=2,X=1);",
        ),
        ("/meter/library.jai", "#module_parameters(X:int=1,Y:int=2);"),
    ] {
        sources
            .insert(Path::new(path), source.as_bytes().to_vec())
            .unwrap();
    }
    ModuleGraph::load_with_provider(
        Path::new("/meter/main.jai"),
        GraphOptions::default(),
        &sources,
    )
    .unwrap()
}

#[test]
fn metered_and_convenience_paths_share_exact_version_four_encoding() {
    let graph = graph();
    let file = graph.module(graph.imports()[0].module()).unwrap().entry();
    let expected = graph.module_environment_origin(file).unwrap();
    let mut work = 0usize;
    let mut metadata = 0usize;
    let actual = graph
        .module_environment_origin_with_work(file, &mut |w, b| {
            work += w;
            metadata += b;
            Ok::<_, ()>(())
        })
        .unwrap();
    assert_eq!(expected, actual);
    assert!(actual.starts_with(b"jai-module-environment\0\x04"));
    assert!(metadata >= actual.capacity() && work > actual.len());
    let other = graph.module(graph.imports()[1].module()).unwrap().entry();
    assert_ne!(actual, graph.module_environment_origin(other).unwrap());
}

#[test]
fn denial_precedes_output_and_traversal_table_allocation() {
    let graph = graph();
    let file = graph.module(graph.imports()[0].module()).unwrap().entry();
    let mut calls = 0;
    let denied = graph.module_environment_origin_with_work(file, &mut |_, bytes| {
        calls += 1;
        if bytes > 0 {
            Err("quota")
        } else {
            Ok(())
        }
    });
    assert_eq!(denied, Err(SourceOriginAdmissionError::Admission("quota")));
    assert_eq!(calls, 2);
    assert!(graph.module_environment_origin(file).is_ok());
}

#[test]
fn late_work_denial_keeps_original_graph_and_fresh_encoding_stable() {
    let graph = graph();
    let file = graph.module(graph.imports()[0].module()).unwrap().entry();
    let expected = graph.module_environment_origin(file).unwrap();
    let files = graph.files().len();
    let mut remaining = 100usize;
    let denied = graph.module_environment_origin_with_work(file, &mut |work, _| {
        remaining = remaining.checked_sub(work).ok_or("fuel")?;
        Ok(())
    });
    assert_eq!(denied, Err(SourceOriginAdmissionError::Admission("fuel")));
    assert_eq!(graph.files().len(), files);
    assert_eq!(graph.module_environment_origin(file).unwrap(), expected);
}
