//! Private finalized runtime options execute on both real backend targets.
#[path = "support/backend_execution.rs"]
mod backend;

#[test]
fn private_runtime_configuration_executes_native_and_wasm_at_o0_and_o2() {
    backend::execute(
        "runtime-config",
        "configuration_check",
        include_str!("../../../tests/compiler-semantics/runtime-configuration/main.jai"),
        &[],
        Some(65536),
    );
}
