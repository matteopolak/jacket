//! Actual native/WASM O0/O2 execution of opaque scalar pointer constants and relocations.
mod common;
use common::run;
use jaic::sema::{Compiler, Options, ProgramSource, TargetCpu, TargetOs, VirtualFs};
use jaic_llvm::{OptLevel, emit_object};
use std::{path::PathBuf, process::Command, rc::Rc};

#[test]
fn raw_pointers_execute_native_and_wasm_at_o0_and_o2() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    for wasm in [false, true] {
        for level in [OptLevel::O0, OptLevel::O2] {
            let dir = root.join(format!("target/raw-pointer-constants/{wasm}-{level:?}"));
            std::fs::create_dir_all(&dir).unwrap();
            let mut options = Options::host();
            options.runtime_support = false;
            if wasm {
                options.os = TargetOs::Wasm;
                options.cpu = TargetCpu::Wasm;
            }
            let mut c = Compiler::new(options, Rc::new(VirtualFs::default()));
            if let Err(e) = c.compile_sources(&[ProgramSource::String(
                include_str!("../../../tests/compiler/semantics/raw-pointer-constants.jai").into(),
            )]) {
                panic!("{}", e.render(&c.sources));
            }
            let object = dir.join("pointers.o");
            let ir = dir.join("pointers.ll");
            emit_object(
                &c.program,
                &jaic_llvm::Options {
                    target: wasm.then(jaic::target::Target::wasm_module),
                    opt_level: level,
                    emit_ir: Some(ir.clone()),
                    ..Default::default()
                },
                &object,
            )
            .unwrap();
            assert!(
                std::fs::read_to_string(ir)
                    .unwrap()
                    .contains("@opaque_handle_value")
            );
            let global = c
                .program
                .globals
                .iter()
                .find(|g| g.name == "exported_handle")
                .unwrap();
            assert_eq!(global.init, 0x80000001u64.to_le_bytes());
            assert!(
                global.relocs.is_empty(),
                "raw constants must be literal pointer bits, not relocations to invented storage"
            );
            if wasm {
                let module = dir.join("pointers.wasm");
                run(Command::new("wasm-ld")
                    .args([
                        "-mwasm64",
                        "--no-entry",
                        "--export=raw_pointer_check",
                        "--export-memory",
                        "--allow-undefined",
                    ])
                    .arg(&object)
                    .arg("-o")
                    .arg(&module));
                run(Command::new("node").args(["-e", r#"
                    const fs=require('fs');
                    const module=new WebAssembly.Module(fs.readFileSync(process.argv[1]));
                    const env={}; let memory;
                    for(const i of WebAssembly.Module.imports(module)) {
                        if(i.kind!=='function')throw Error('unexpected import');
                        env[i.name]=()=>{throw Error('unexpected runtime failure '+i.name);};
                    }
                    env.memcpy=(d,s,n)=>{new Uint8Array(memory.buffer).copyWithin(Number(d),Number(s),Number(s+n));return d;};
                    env.memset=(d,v,n)=>{new Uint8Array(memory.buffer).fill(Number(v),Number(d),Number(d+n));return d;};
                    const instance=new WebAssembly.Instance(module,{env}); memory=instance.exports.memory;
                    const result=instance.exports.raw_pointer_check();
                    if(result!==0)throw Error('raw pointer failure '+result);
                "#]).arg(&module));
            } else {
                let driver = dir.join("driver.c");
                std::fs::write(&driver,"#include <stdint.h>\nextern int32_t raw_pointer_check(void); extern uint64_t opaque_handle_value(void); int main(void){if(opaque_handle_value()!=UINT64_C(0x80000001))return 99;return raw_pointer_check();}\n").unwrap();
                let executable = dir.join(format!("driver{}", std::env::consts::EXE_SUFFIX));
                run(Command::new("clang")
                    .args(["-std=c11", "-Wall", "-Wextra", "-Werror"])
                    .arg(&driver)
                    .arg(&object)
                    .arg("-o")
                    .arg(&executable));
                run(&mut Command::new(&executable));
            }
        }
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
    let error = emit_object(
        &c.program,
        &jaic_llvm::Options {
            target: Some(jaic::target::Target::parse("wasm32-unknown-unknown").unwrap()),
            ..Default::default()
        },
        &PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("raw-pointer-constants/unsupported.o"),
    )
    .unwrap_err();
    assert!(
        error.contains("unsupported target architecture"),
        "wasm32: {error}"
    );
}
