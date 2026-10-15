//! Actual O0/O2 native and WASM execution of explicit word-sized shift operands.
#[path = "support/backend_execution.rs"]
mod backend_execution;

#[test]
fn explicit_cast_shift_widths_execute_native_and_wasm() {
    backend_execution::execute(
        "cast-shift-widths",
        "cast_shift_check",
        include_str!("../../../tests/compiler-semantics/cast-shift-widths.jai"),
        &[],
        None,
    );
}
