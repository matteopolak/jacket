//! Native and WASM execution agrees with folded integer constants.
#[path = "support/backend_execution.rs"]
mod backend_execution;
mod common;

#[test]
fn compiled_integer_arithmetic_matches_folded_constants() {
    backend_execution::execute(
        "integer-constant-arithmetic",
        "integer_constant_arithmetic_check",
        include_str!("../../../tests/compiler/semantics/integer-constant-arithmetic.jai"),
        &[],
        None,
    );
}
