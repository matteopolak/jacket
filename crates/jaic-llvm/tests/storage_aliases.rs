//! Real C-owned variables shared through filtered reexports, native and WASM64 O0/O2.
use jaic::sema::{Compiler, NativeFs};
use std::rc::Rc;
use test_support::TempDir;
use test_support::backend::{self, VARIANTS};

#[test]
fn filtered_reexports_share_actual_c_memory_native_and_wasm_o0_o2() {
    if !backend::tools_available() {
        return;
    }
    let fixture = test_support::repo::path("tests/compiler/semantics/storage-aliases");
    let provider = fixture.join("provider.c");
    for variant in VARIANTS {
        let dir = TempDir::new(&format!("storage-aliases-backend-{}", variant.label()));
        let mut options = variant.options();
        options.import_paths.push(fixture.clone());
        let mut c = Compiler::new(options, Rc::new(NativeFs));
        if let Err(e) = c.compile_program(&fixture.join("main.jai")) {
            panic!("{}", e.render(&c.src.sources));
        }
        assert_eq!(
            c.out.program.foreigns.iter().filter(|f| f.is_data).count(),
            2
        );
        assert!(!c.out.program.globals.iter().any(|g| {
            ["storage_alias_counter", "storage_alias_pair"].contains(&g.name.as_str())
        }));
        let object = dir.join("aliases.o");
        variant.emit(&c, &object, None);
        // The check runs twice: the C variables keep their state between the calls.
        let exports = ["storage_alias_check", "storage_alias_check"];
        let wasm_provider = dir.join("provider.o");
        if variant.wasm {
            backend::compile_wasm_c(&provider, &wasm_provider);
        }
        backend::check(
            &variant,
            &dir,
            &object,
            &exports,
            None,
            &[&provider],
            &[&wasm_provider],
        );
    }
}
