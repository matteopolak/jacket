//! Enum dependency runtime regression on actual host and WASM64 backends.
use test_support::TempDir;
use test_support::backend::{self, VARIANTS};

#[test]
fn enum_dependencies_execute_native_and_wasm_at_o0_and_o2() {
    if !backend::tools_available() {
        return;
    }
    let source = include_str!("../../../tests/compiler/semantics/enum-dependencies.jai");
    for variant in VARIANTS {
        let dir = TempDir::new(&format!("enum-dependencies-{}", variant.label()));
        let compiler = variant.compile(source);
        let object = dir.join("enums.o");
        variant.emit(&compiler, &object, None);
        backend::check(
            &variant,
            &dir,
            &object,
            &["enum_dependency_check"],
            None,
            &[],
            &[],
        );
    }
}
