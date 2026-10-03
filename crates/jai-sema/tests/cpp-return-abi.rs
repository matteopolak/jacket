use jai_modules::{GraphOptions, ModuleGraph, SourceOverlay};
use jai_types::ForeignReturnAbi;
use std::path::Path;

fn graph(source: &str) -> ModuleGraph {
    let path = Path::new("/own-cpp-return/main.jai");
    let mut overlay = SourceOverlay::new();
    overlay.insert(path, source.as_bytes().to_vec()).unwrap();
    ModuleGraph::load_with_provider(path, GraphOptions::default(), &overlay).unwrap()
}

#[test]
fn source_prototype_callback_and_named_definition_keep_the_policy() {
    let graph = graph(
        r#"
Pair :: struct { x:float; y:float; }
Callback :: #type (value:float)->Pair #c_call #cpp_return_type_is_non_pod;
make :: (value:float)->Pair #cpp_return_type_is_non_pod #foreign "own_make";
#program_export "own_invoke" invoke :: (callback:Callback,value:float)->Pair #c_call #cpp_return_type_is_non_pod { return callback(value); }
"#,
    );
    let library = jai_sema::resolve_library(&graph).unwrap();
    let prototype = library
        .prototypes()
        .iter()
        .find(|p| p.origin.external_symbol() == Some("own_make"))
        .unwrap();
    assert_eq!(
        library
            .types()
            .procedure_definition(prototype.signature)
            .unwrap()
            .return_abi,
        ForeignReturnAbi::CppNonPod
    );
    let procedure = library
        .procedures()
        .iter()
        .find(|procedure| {
            let signature = library
                .types()
                .procedure_definition(procedure.signature)
                .unwrap();
            signature.return_abi == ForeignReturnAbi::CppNonPod && signature.parameters.len() == 2
        })
        .expect("actual named invoke definition");
    let signature = library
        .types()
        .procedure_definition(procedure.signature)
        .unwrap();
    assert_eq!(signature.return_abi, ForeignReturnAbi::CppNonPod);
    assert_eq!(
        library
            .types()
            .procedure_definition(signature.parameters[0])
            .unwrap()
            .return_abi,
        ForeignReturnAbi::CppNonPod
    );
}

#[test]
fn equal_logical_callback_shapes_cannot_erase_result_abi() {
    let source = r#"
Pair :: struct { x:float; y:float; }
make :: (value:float)->Pair #cpp_return_type_is_non_pod #foreign "own_make";
main :: ()->int { callback:(value:float)->Pair #c_call = make; return 0; }
"#;
    assert!(jai_sema::resolve_graph(&graph(source)).is_err());
}
