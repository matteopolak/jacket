//! Native and WASM execution agrees with folded integer constants.
use test_support::backend as backend_execution;

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
