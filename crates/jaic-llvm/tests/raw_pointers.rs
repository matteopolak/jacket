//! Actual native/WASM O0/O2 execution of opaque scalar pointer constants and relocations.
use jaic::sema::{Compiler, Options, ProgramSource, VirtualFs};
use jaic_llvm::emit_object;
use std::rc::Rc;
use test_support::TempDir;
use test_support::backend::{self, VARIANTS};
use test_support::ir;

#[test]
fn raw_pointers_execute_native_and_wasm_at_o0_and_o2() {
    if !backend::tools_available() {
        return;
    }
    let source = include_str!("../../../tests/compiler/semantics/raw-pointer-constants.jai");
    for variant in VARIANTS {
        let dir = TempDir::new(&format!("raw-pointer-constants-{}", variant.label()));
        let compiler = variant.compile(source);
        let object = dir.join("pointers.o");
        let ir_path = dir.join("pointers.ll");
        variant.emit(&compiler, &object, Some(&ir_path));
        ir::assert_contains(
            &std::fs::read_to_string(ir_path).unwrap(),
            "@opaque_handle_value",
        );
        let global = compiler
            .out
            .program
            .globals
            .iter()
            .find(|g| g.name == "exported_handle")
            .unwrap();
        assert_eq!(global.init, 0x8000_0001_u64.to_le_bytes());
        assert!(
            global.relocs.is_empty(),
            "raw constants must be literal pointer bits, not relocations to invented storage"
        );
        let driver = "#include <stdint.h>\nextern int32_t raw_pointer_check(void); extern uint64_t opaque_handle_value(void); int main(void){if(opaque_handle_value()!=UINT64_C(0x80000001))return 99;return raw_pointer_check();}\n";
        backend::check(
            &variant,
            &dir,
            &object,
            &["raw_pointer_check"],
            Some(driver),
            &[],
            &[],
        );
    }
}

#[test]
fn unsupported_32_bit_targets_do_not_masquerade_as_supported_pointer_models() {
    let mut options = Options::host();
    options.runtime_support = false;
    let mut c = Compiler::new(options, Rc::new(VirtualFs::default()));
    c.compile_sources(&[ProgramSource::String("main :: () {}".into())])
        .unwrap();
    // A 32-bit x86 triple is not a target at all; wasm32 is one the backend refuses.
    for target in ["i686-pc-windows-msvc", "i686-unknown-linux-gnu"] {
        let error = jaic::target::Target::parse(target).unwrap_err();
        assert!(
            error.contains("unsupported target architecture"),
            "{target}: {error}"
        );
    }
    let dir = TempDir::new("raw-pointer-unsupported");
    let error = emit_object(
        &c.out.program,
        &jaic_llvm::Options {
            target: Some(jaic::target::Target::parse("wasm32-unknown-unknown").unwrap()),
            ..Default::default()
        },
        &dir.join("unsupported.o"),
    )
    .unwrap_err();
    assert!(
        error.contains("unsupported target architecture"),
        "wasm32: {error}"
    );
}
