use jai_modules::{GraphOptions, ModuleGraph, SourceOverlay};
use jai_sema::resolve_graph;
use std::path::Path;

fn error_for(text: &str) -> (ModuleGraph, jai_source::LocatedDiagnostic) {
    let path = Path::new("/jai-statement-spans/main.jai");
    let mut sources = SourceOverlay::new();
    sources.insert(path, text.as_bytes().to_vec()).unwrap();
    let graph = ModuleGraph::load_with_provider(path, GraphOptions::default(), &sources).unwrap();
    let error = resolve_graph(&graph).unwrap_err();
    (graph, error)
}

#[test]
fn assignment_name_errors_retain_the_complete_statement_source_range() {
    let text = "\u{feff}// é keeps original byte offsets\r\nmain :: () {\r\n value := 3;\r\n missing = value;\r\n}";
    let (graph, error) = error_for(text);
    let source = graph.sources().get(error.location.source).unwrap();
    assert_eq!(source.path(), Path::new("/jai-statement-spans/main.jai"));
    assert_eq!(error.location.span.text(source.text()), "missing = value;");
    assert!(error.render(graph.sources()).contains("main.jai:4:2:"));
}

#[test]
fn unreachable_statement_errors_identify_the_statement_after_the_return() {
    let text = "main :: () { return;\n value := 3; }";
    let (graph, error) = error_for(text);
    assert_eq!(error.message, "unreachable statement");
    assert_eq!(
        error
            .location
            .span
            .text(graph.sources().get(error.location.source).unwrap().text()),
        "value := 3;"
    );
}

#[test]
fn nested_lowering_restores_the_containing_statement_diagnostic_context() {
    let text = "main :: () { { local := 1; }\n missing = 2; }";
    let (graph, error) = error_for(text);
    assert_eq!(
        error
            .location
            .span
            .text(graph.sources().get(error.location.source).unwrap().text()),
        "missing = 2;"
    );
}
