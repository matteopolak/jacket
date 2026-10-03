use jai_modules::{GraphOptions, ModuleGraph, SourceOverlay};
use std::path::Path;
fn graph(source: &str) -> ModuleGraph {
    let path = Path::new("/own-mixed-results/main.jai");
    let mut overlay = SourceOverlay::new();
    overlay.insert(path, source.as_bytes().to_vec()).unwrap();
    ModuleGraph::load_with_provider(path, GraphOptions::default(), &overlay).unwrap()
}
#[test]
fn mixed_binding_rejects_discarding_required_results() {
    let error = jai_sema::resolve_graph(&graph(
        "pair :: ()->s32 #must,s32 {return 1,2;} main :: ()->int {_=,new := pair();return new;}",
    ))
    .unwrap_err();
    assert!(error.message.contains("#must"), "{error:?}");
}
#[test]
fn new_bindings_are_not_visible_during_result_evaluation() {
    let error = jai_sema::resolve_graph(&graph(
        "main :: ()->int {old:s32=0;old=,new := 1,new;return old;}",
    ))
    .unwrap_err();
    assert!(error.message.contains("unknown"), "{error:?}");
}
#[test]
fn captured_callback_contract_reaches_a_new_mixed_binding() {
    let error=jai_sema::resolve_graph(&graph("callback :: ()->s32 #must {return 42;} pair :: ()->s32,()->s32 #must {return 0,callback;} main :: ()->int {old:s32=0;old=,new := pair();new();return 0;}")).unwrap_err();
    assert!(error.message.contains("#must"), "{error:?}");
}
