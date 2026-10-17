//! Actual native and WASM execution, at both optimization levels.
#[path = "support/backend_execution.rs"]
mod backend;
mod common;
#[path = "../../jaic/tests/support/named_while_source.rs"]
mod fixture;

#[test]
fn caller_visible_while_condition_executes_native_and_wasm() {
    backend::execute(
        "named-while",
        "named_while_check",
        fixture::SOURCE,
        &[],
        None,
    );
}
