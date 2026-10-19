//! The fiber routines' module assembly (`jaic::fiber::module_asm`) assembles for every native
//! target, whichever machine runs the test.
use inkwell::OptimizationLevel;
use inkwell::context::Context;
use inkwell::targets::{
    CodeModel, FileType, InitializationConfig, RelocMode, Target, TargetTriple,
};
use jaic::abi::Arch;

#[test]
fn the_module_assembly_assembles_for_every_target() {
    Target::initialize_all(&InitializationConfig::default());
    let cases = [
        ("x86_64-apple-darwin", Arch::X86_64, true),
        ("arm64-apple-darwin", Arch::Aarch64, true),
        ("x86_64-unknown-linux-gnu", Arch::X86_64, false),
        ("aarch64-unknown-linux-gnu", Arch::Aarch64, false),
        ("x86_64-pc-windows-msvc", Arch::Win64, false),
        ("aarch64-pc-windows-msvc", Arch::Win64Arm, false),
    ];
    for (triple, arch, macho) in cases {
        let triple = TargetTriple::create(triple);
        let Ok(target) = Target::from_triple(&triple) else {
            eprintln!("skipping {triple}: this LLVM was built without it");
            continue;
        };
        let machine = target
            .create_target_machine(
                &triple,
                "generic",
                "",
                OptimizationLevel::None,
                RelocMode::PIC,
                CodeModel::Default,
            )
            .unwrap();
        let context = Context::create();
        let module = context.create_module("fiber");
        module.set_triple(&triple);
        module.set_inline_assembly(&jaic::fiber::module_asm(arch, macho).unwrap());
        let object = machine
            .write_to_memory_buffer(&module, FileType::Object)
            .unwrap_or_else(|e| panic!("{triple} does not assemble: {e}"));
        assert!(!object.as_slice().is_empty(), "{triple} produced no object");
    }
}
