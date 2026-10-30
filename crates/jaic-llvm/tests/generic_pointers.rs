//! Actual native/WASM O0/O2 execution of default generic pointer binding and identity.
use test_support::TempDir;
use test_support::backend::{self, VARIANTS};

#[test]
fn default_generic_pointers_execute_native_and_wasm_at_o0_and_o2() {
    if !backend::tools_available() {
        return;
    }
    let source = include_str!("../../../tests/compiler/semantics/default-generic-pointers.jai");
    for variant in VARIANTS {
        let dir = TempDir::new(&format!("default-generic-pointers-{}", variant.label()));
        let compiler = variant.compile(source);
        let object = dir.join("pointers.o");
        variant.emit(&compiler, &object, None);
        backend::check(
            &variant,
            &dir,
            &object,
            &["generic_pointer_check"],
            None,
            &[],
            &[],
        );
    }
}
