use jai_modules::{GraphOptions, ModuleGraph, SourceOverlay};
use std::path::Path;
fn graph(source: &str) -> ModuleGraph {
    let path = Path::new("/own-cpp-method/main.jai");
    let mut overlay = SourceOverlay::new();
    overlay.insert(path, source.as_bytes().to_vec()).unwrap();
    ModuleGraph::load_with_provider(path, GraphOptions::default(), &overlay).unwrap()
}
#[test]
fn foreign_cpp_method_cannot_execute_in_the_vm() {
    let program = jai_sema::resolve_graph(&graph("method :: (this:*s32)->s32 #cpp_method #foreign \"own_method\"; main :: ()->int {value:s32=42;return method(*value);} ")).unwrap();
    assert!(matches!(
        jai_vm::execute(&program, jai_vm::Limits::default()).outcome,
        jai_vm::Outcome::Failed(jai_vm::Error::UnsupportedForeignProcedure(_))
    ));
}
#[test]
fn c_and_cpp_method_callbacks_are_not_interchangeable() {
    let error = jai_sema::resolve_graph(&graph("method :: (this:*void)->s32 #cpp_method {return 42;} main :: ()->int {callback:(this:*void)->s32 #c_call = method;return 0;}")).unwrap_err();
    assert!(error.message.contains("procedure"), "{error:?}");
}
