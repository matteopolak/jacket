use jai_modules::{GraphOptions, ModuleGraph, SourceOverlay};
use std::path::Path;
fn rejection(source: &str) -> String {
    let path = Path::new("/own-constant-results/main.jai");
    let mut overlay = SourceOverlay::new();
    overlay.insert(path, source.as_bytes().to_vec()).unwrap();
    let graph = ModuleGraph::load_with_provider(path, GraphOptions::default(), &overlay).unwrap();
    jai_sema::resolve_graph(&graph).unwrap_err().message
}
#[test]
fn result_count_must_match_the_declaration_group() {
    let error = rejection(
        "facts :: ()->s32,s32 {return 1,2;} main :: ()->int {a,b,c :: #run facts();return a;}",
    );
    assert!(
        error.contains("returns 2 results") && error.contains("3 result destinations"),
        "{error}"
    );
}
#[test]
fn duplicate_names_reject_before_initializer_execution() {
    let error = rejection(
        "facts :: ()->s32,s32 {return 1,2;} main :: ()->int {a,a :: #run facts();return a;}",
    );
    assert!(error.contains("duplicate"), "{error}");
}
#[test]
fn discarded_required_constant_result_is_rejected() {
    let error = rejection(
        "facts :: ()->s32 #must,s32 {return 1,2;} main :: ()->int {_,answer :: #run facts();return answer;}",
    );
    assert!(error.contains("#must"), "{error}");
}
