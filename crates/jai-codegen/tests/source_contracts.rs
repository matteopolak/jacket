use jai_codegen::{native_reachability::Publication, target::NativeTarget};
use jai_modules::{GraphOptions, ModuleGraph, SourceOverlay};
use std::path::Path;
#[test]
fn object_publication_emits_jai_pack_extern_without_claiming_a_linked_provider() {
    let path = Path::new("/own-source-contract-llvm/main.jai");
    let mut overlay = SourceOverlay::new();
    overlay.insert(path,b"join :: (values:..string,separator:=\"\",before_first:=false,after_last:=false)->string #foreign; caller :: ()->string{return join(\"a\",\"b\");}".to_vec()).unwrap();
    let graph = ModuleGraph::load_with_provider(path, GraphOptions::default(), &overlay).unwrap();
    let library = jai_sema::resolve_library(&graph).unwrap();
    let target = NativeTarget::new().unwrap();
    let context = jai_codegen::Context::create();
    let module =
        jai_codegen::lower_library_for_target(&context, &library, &Publication::AllBodies, &target)
            .unwrap();
    module.verify().unwrap();
    let external = module.get_function("join").unwrap();
    assert!(external.get_first_basic_block().is_none());
    assert!(!external.get_type().is_var_arg());
    assert_eq!(external.count_params(), 5);
    assert!(external.get_nth_param(0).unwrap().is_pointer_value());
    assert!(external.get_nth_param(1).unwrap().is_struct_value());
    assert!(
        external
            .get_type()
            .get_return_type()
            .unwrap()
            .is_struct_type()
    );
}

#[test]
fn application_object_keeps_extern_but_executable_requires_provider() {
    let path = Path::new("/own-source-contract-program/main.jai");
    let mut overlay = SourceOverlay::new();
    overlay.insert(path, b"join :: (values:..string,separator:=\"\")->string #foreign; main :: ()->s64 { join(\"a\",\"b\"); return 42; }".to_vec()).unwrap();
    let graph = ModuleGraph::load_with_provider(path, GraphOptions::default(), &overlay).unwrap();
    let program = jai_sema::resolve_graph(&graph).unwrap();
    let target = NativeTarget::new().unwrap();
    let context = jai_codegen::Context::create();
    let module = jai_codegen::lower_program_object_for_target(&context, &program, &target).unwrap();
    module.verify().unwrap();
    assert!(
        module
            .get_function("join")
            .unwrap()
            .get_first_basic_block()
            .is_none()
    );
    assert!(matches!(
        jai_codegen::lower_for_target(&context, &program, &target),
        Err(jai_codegen::Error::Reachability(
            jai_codegen::native_reachability::Error::SourceContract { .. }
        ))
    ));
}
