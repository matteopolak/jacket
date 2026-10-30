//! Disabling runtime support must retain explicit native null guards.
use jaic::sema::{Compiler, Options, ProgramSource, VirtualFs};
use jaic_llvm::{OptLevel, emit_object};
use std::{process::Command, rc::Rc};
use test_support::{TempDir, ir, process, toolchain};

#[test]
fn null_guards_trap_without_a_runtime_reporter() {
    let Some(clang) = toolchain::clang() else {
        return;
    };
    let source = "#program_export \"guarded_read\"\nread :: (pointer: *s32) -> s32 #c_call { return pointer.*; }\n";
    for level in [OptLevel::O0, OptLevel::O2] {
        let mut options = Options::host();
        options.runtime_support = false;
        options.stack_trace = false;
        let mut compiler = Compiler::new(options, Rc::new(VirtualFs::default()));
        compiler
            .compile_sources(&[ProgramSource::String(source.into())])
            .unwrap();
        assert!(compiler.out.program.check_failed.is_none());
        let directory = TempDir::new(&format!("runtime-disabled-{level:?}"));
        let object = directory.join("guard.o");
        let ir = directory.join("guard.ll");
        emit_object(
            &compiler.out.program,
            &jaic_llvm::Options {
                opt_level: level,
                emit_ir: Some(ir.clone()),
                ..Default::default()
            },
            &object,
        )
        .unwrap();
        let ir = std::fs::read_to_string(ir).unwrap();
        ir::assert_contains(&ir, "jaic.null_fail");
        ir::assert_contains(&ir, "call void @llvm.trap");
        let driver = directory.join("driver.c");
        std::fs::write(&driver, "#include <stdint.h>\n#include <stdio.h>\nextern int32_t guarded_read(int32_t *);\nint main(void) { int32_t v=42; if (guarded_read(&v)!=42) return 0; puts(\"valid\"); fflush(stdout); return guarded_read(NULL); }\n").unwrap();
        let executable = directory.join(format!("guard{}", std::env::consts::EXE_SUFFIX));
        process::run(
            Command::new(&clang)
                .arg(driver)
                .arg(object)
                .arg("-o")
                .arg(&executable),
        );
        let output = process::output(&mut Command::new(executable));
        assert_eq!(output.stdout, b"valid\n");
        assert!(!output.status.success(), "null access returned normally");
    }
}
