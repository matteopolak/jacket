use jai_modules::{GraphOptions, ModuleGraph, SourceOverlay};
use jai_sema::{PreparedLibrarySession, ResolveOptions, SourcePrefixReadiness};
use jai_source::SourceSpan;
use std::path::Path;

#[test]
fn no_progress_keeps_the_actual_cyclic_constant_demand_and_phase() {
    let source =
        "count::()->int #no_context{return Count;} Count::count(); Alias::#type [Count]u8;";
    let path = Path::new("/prepared-no-progress/main.jai");
    let mut provider = SourceOverlay::new();
    provider.insert(path, source.as_bytes().to_vec()).unwrap();
    let graph = ModuleGraph::load_with_provider(path, GraphOptions::default(), &provider).unwrap();
    let count = graph
        .declarations()
        .iter()
        .find(|declaration| graph.symbols().name(declaration.name()) == "Count")
        .unwrap();
    let jai_syntax::FileDeclarationKind::Constant(constant) = &count.syntax().kind else {
        panic!("the actual Count producer is a constant")
    };
    let location = SourceSpan {
        source: count.location().source,
        span: constant.initializer.span,
    };
    let mut session = PreparedLibrarySession::new(&graph, &ResolveOptions::default()).unwrap();
    let SourcePrefixReadiness::Failed(error) = session.drive_source_prefix(&mut jai_vm::NoEffects)
    else {
        panic!("a cyclic checked constant cannot publish a ready type")
    };
    assert_eq!(error.location, location);
    assert_eq!(error.location.span.text(source), "count()");
    assert!(
        error
            .message
            .contains("unresolved or cyclic suspended #run dependencies"),
        "{error}"
    );
    assert!(error.message.contains("Procedure("), "{error}");
}
