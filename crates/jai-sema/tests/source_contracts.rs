use jai_modules::{GraphOptions, ModuleGraph, SourceOverlay};
use jai_types::{CallingConvention, ContextMode, Variadic};
use std::path::Path;
fn graph(source: &str) -> ModuleGraph {
    let path = Path::new("/own-source-contract/main.jai");
    let mut overlay = SourceOverlay::new();
    overlay.insert(path, source.as_bytes().to_vec()).unwrap();
    ModuleGraph::load_with_provider(path, GraphOptions::default(), &overlay).unwrap()
}
#[test]
fn bare_contract_retains_true_jai_abi_and_has_no_native_library_authority() {
    let library=jai_sema::resolve_library(&graph("join :: (values:..string, separator:=\"\", before_first:=false, after_last:=false)->string #foreign;")).unwrap();
    let prototype = &library.prototypes()[0];
    assert!(
        matches!(&prototype.origin,jai_sema::PrototypeOrigin::SourceContract{symbol} if symbol=="join")
    );
    let signature = library
        .types()
        .procedure_definition(prototype.signature)
        .unwrap();
    assert_eq!(signature.convention, CallingConvention::Jai);
    assert_eq!(signature.context, ContextMode::Implicit);
    assert_eq!(signature.parameters.len(), 4);
    assert!(matches!(
        signature.variadic,
        Variadic::Jai {
            parameter: 0,
            ..
        }
    ));
    assert!(library.foreign_libraries().is_empty());
}
#[test]
fn call_binds_a_pack_and_default_but_cannot_execute_without_provider() {
    let program = jai_sema::resolve_graph_with_options(&graph(
        "consume :: (values:..s32,extra:s32=2)->s32 #foreign; main :: ()->int{return consume(40);}",
    ), &jai_sema::ResolveOptions { layout: Some(jai_types::LayoutPolicy::lp64()), ..jai_sema::ResolveOptions::default() }, &mut jai_vm::NoEffects)
    .unwrap();
    let outcome = jai_vm::execute(&program, jai_vm::Limits::default()).outcome;
    assert!(
        matches!(
            outcome,
            jai_vm::Outcome::Failed(jai_vm::Error::InvalidIr(
                "source contract has no compile-time implementation provider"
            ))
        ),
        "{outcome:?}"
    );
}
