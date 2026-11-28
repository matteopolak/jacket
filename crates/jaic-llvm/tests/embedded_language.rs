//! The language support embedded programs use (docs/extensions/embedded.md): `@Section` on
//! globals and procedures, and volatile loads and stores that the optimizer keeps.
use std::rc::Rc;

use jaic::sema::{Compiler, Options, ProgramSource, VirtualFs};
use jaic::target::{TargetCpu, TargetOs};
use jaic_llvm::{Codegen, OptLevel, emit_object};
use test_support::TempDir;
use test_support::ir::{body as function, has_line};

const SOURCE: &str = r#"
volatile_load :: (pointer: *$T) -> T #intrinsic;
volatile_store :: (pointer: *$T, value: T) #intrinsic;

vectors: [4] u32 = .[1, 2, 3, 4]; @Section(".vector_table")

plain: [4] u32 = .[5, 6, 7, 8];

// Whole-module assembly: the symbols it defines are the program's, like the ones a linker
// script or a board names.
reset_stub :: ".global stub_entry\nstub_entry:\n    ret\n"; @Module_Asm

stub_entry :: () #elsewhere #c_call;

#program_export "calls_stub" calls_stub :: () #c_call { stub_entry(); }

#program_export "boot" boot :: () #c_call { } @Section(".text.boot")

#program_export "ordinary" ordinary :: () #c_call { }

// Two reads of one address are two loads, and a read whose result is unused still happens.
#program_export "poll" poll :: (register: *u32) -> u32 #c_call {
    first := volatile_load(register);
    volatile_load(register);
    second := volatile_load(register);
    return first + second;
}

#program_export "write_twice" write_twice :: (register: *u32) #c_call {
    volatile_store(register, 1);
    volatile_store(register, 2);
}

#program_export "narrow" narrow :: (register: *u8) -> u8 #c_call { return volatile_load(register); }

#program_export "plain_twice" plain_twice :: (register: *u32) -> u32 #c_call {
    return register.* + register.*;
}
"#;

fn optimized_ir() -> String {
    let dir = TempDir::new("embedded-language");
    let mut options = Options::host();

    options.os = TargetOs::None;
    options.cpu = TargetCpu::Arm64;
    options.runtime_support = false;
    options.stack_trace = false;
    let mut compiler = Compiler::new(options, Rc::new(VirtualFs::default()));

    if let Err(e) = compiler.compile_sources(&[ProgramSource::String(SOURCE.into())]) {
        panic!("{}", e.render(&compiler.src.sources));
    }

    let options = jaic_llvm::Options {
        target: Some(jaic::target::Target::parse("aarch64-none-elf").unwrap()),
        opt_level: OptLevel::O2,
        codegen: Codegen {
            single_unit: true,
            ir_after: Some(dir.join("optimized.ll")),
            ..Codegen::default()
        },
        ..Default::default()
    };

    emit_object(&compiler.out.program, &options, &dir.join("embedded.o")).unwrap();
    std::fs::read_to_string(dir.join("optimized.ll")).unwrap()
}

#[test]
fn a_section_note_places_a_global_and_a_procedure() {
    let ir = optimized_ir();

    assert!(
        has_line(&ir, &["@vectors", "section \".vector_table\""]),
        "{ir}"
    );
    assert!(!has_line(&ir, &["@plain", "section"]), "{ir}");
    let boot = function(&ir, "boot");

    assert!(boot.contains("section \".text.boot\""), "{boot}");
    assert!(!function(&ir, "ordinary").contains("section"), "{ir}");
}

#[test]
fn volatile_accesses_are_kept_one_for_one() {
    let ir = optimized_ir();
    let poll = function(&ir, "poll");

    assert_eq!(poll.matches("load volatile i32").count(), 3, "{poll}");
    let writes = function(&ir, "write_twice");

    assert_eq!(writes.matches("store volatile i32").count(), 2, "{writes}");
    assert!(function(&ir, "narrow").contains("load volatile i8"), "{ir}");
    // The same two reads through an ordinary pointer are merged.
    let plain = function(&ir, "plain_twice");

    assert_eq!(plain.matches("load i32").count(), 1, "{plain}");
    assert!(!plain.contains("volatile"), "{plain}");
}

#[test]
fn a_module_asm_note_adds_assembly_to_the_module() {
    let ir = optimized_ir();

    assert!(ir.contains("module asm"), "{ir}");
    assert!(ir.contains("stub_entry:"), "{ir}");
    assert!(function(&ir, "calls_stub").contains("@stub_entry"), "{ir}");
}
