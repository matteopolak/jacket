use jai_modules::{Binding, GraphError, GraphOptions, ModuleGraph, SourceOverlay};
use jai_source::SourceRecordKind;
use jai_syntax::NamePath;
use std::path::Path;

fn graph(root: &str, files: &[(&str, &str)]) -> Result<ModuleGraph, GraphError> {
    let mut inputs = SourceOverlay::new();
    inputs
        .insert(Path::new("/strings/main.jai"), root.as_bytes().to_vec())
        .unwrap();
    for (path, text) in files {
        inputs
            .insert(Path::new(path), text.as_bytes().to_vec())
            .unwrap();
    }
    ModuleGraph::load_with_provider(
        Path::new("/strings/main.jai"),
        GraphOptions::default(),
        &inputs,
    )
}
fn lookup(graph: &ModuleGraph, names: &[&str]) -> Binding {
    let mut names = names.iter().map(|name| graph.symbols().find(name).unwrap());
    graph
        .lookup(
            graph.module(graph.root()).unwrap().entry(),
            &NamePath {
                root: names.next().unwrap(),
                members: names.collect(),
            },
        )
        .unwrap()
}

#[test]
fn literal_sites_retain_distinct_nominal_owners_and_parameter_environments() {
    let graph = graph("A::#import,string \"#module_parameters(X:int=1); T::struct{value:int=X;}\"(X=41); B::#import,string \"#module_parameters(X:int=1); T::struct{value:int=X;}\"(X=42);", &[]).unwrap();
    assert_ne!(lookup(&graph, &["A", "T"]), lookup(&graph, &["B", "T"]));
    let first = graph.module(graph.imports()[0].module()).unwrap().entry();
    let second = graph.module(graph.imports()[1].module()).unwrap().entry();
    assert_ne!(
        graph.module_environment_origin(first).unwrap(),
        graph.module_environment_origin(second).unwrap()
    );
    for file in [first, second] {
        let source = graph
            .sources()
            .get(graph.file(file).unwrap().source())
            .unwrap();
        assert!(matches!(source.kind(), SourceRecordKind::Embedded { .. }));
        assert_eq!(source.resolution_path(), Path::new("/strings/main.jai"));
        assert!(source.physical_path().is_none());
        assert_eq!(
            source.importing_site().unwrap().source,
            graph
                .file(graph.module(graph.root()).unwrap().entry())
                .unwrap()
                .source()
        );
    }
}

#[test]
fn relative_load_and_file_import_use_the_original_provider_anchor() {
    let graph = graph(
        r##"A::#import,string "#load \"child.jai\"; B::#import,file \"other.jai\";";"##,
        &[
            ("/strings/child.jai", "answer::42;"),
            ("/strings/other.jai", "value::41;"),
        ],
    )
    .unwrap();
    assert!(matches!(
        lookup(&graph, &["A", "answer"]),
        Binding::Declaration(_)
    ));
    assert!(matches!(
        lookup(&graph, &["A", "B", "value"]),
        Binding::Declaration(_)
    ));
    assert_eq!(graph.loads().len(), 1);
}

#[test]
fn physical_file_with_an_embedded_label_is_a_different_source_and_module() {
    let root = "A::#import,string \"answer::42;\";";
    let first = graph(root, &[]).unwrap();
    let file = first.module(first.imports()[0].module()).unwrap().entry();
    let label = first
        .sources()
        .get(first.file(file).unwrap().source())
        .unwrap()
        .path();
    let root = format!("{root} B::#import,file {:?};", label.to_str().unwrap());
    let graph = graph(&root, &[(label.to_str().unwrap(), "answer::41;")]).unwrap();
    assert_ne!(
        lookup(&graph, &["A", "answer"]),
        lookup(&graph, &["B", "answer"])
    );
    let a = graph.module(graph.imports()[0].module()).unwrap().entry();
    let b = graph.module(graph.imports()[1].module()).unwrap().entry();
    assert_ne!(
        graph.file(a).unwrap().source(),
        graph.file(b).unwrap().source()
    );
    assert_ne!(
        graph.module_environment_origin(a).unwrap(),
        graph.module_environment_origin(b).unwrap()
    );
}

#[test]
fn literal_parse_errors_keep_the_generated_source_location() {
    let error = graph("#import,string \"answer::;\";", &[]).unwrap_err();
    let GraphError::Located {
        diagnostic,
        rendered,
    } = error
    else {
        panic!("{error:?}")
    };
    assert!(diagnostic.location.source.index() > 0);
    assert!(rendered.contains("string-import-"), "{rendered}");
}
