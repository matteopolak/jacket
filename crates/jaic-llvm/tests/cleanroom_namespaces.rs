//! Real native/WASM execution of mixed loaded-file constructor namespaces.
#[path = "support/backend_execution.rs"]
mod backend;
#[path = "../../jaic/tests/support/namespace_source.rs"]
mod source;

#[test]
fn loaded_constructor_namespaces_execute_native_and_wasm_at_o0_and_o2() {
    backend::execute(
        "namespaces",
        "namespace_check",
        source::SOURCE,
        source::FILES,
        None,
    );
}
