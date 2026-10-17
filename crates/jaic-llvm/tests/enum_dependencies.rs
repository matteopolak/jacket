//! Enum dependency runtime regression on actual host and WASM64 backends.
mod common;
use common::run;
use jaic::sema::{Compiler, Options, ProgramSource, TargetCpu, TargetOs, VirtualFs};
use jaic_llvm::{OptLevel, emit_object};
use std::{path::PathBuf, process::Command, rc::Rc};

#[test]
fn enum_dependencies_execute_native_and_wasm_at_o0_and_o2() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    for wasm in [false, true] {
        for level in [OptLevel::O0, OptLevel::O2] {
            let dir = root.join(format!("target/enum-dependencies/{wasm}-{level:?}"));
            std::fs::create_dir_all(&dir).unwrap();
            let mut options = Options::host();
            options.runtime_support = false;
            if wasm {
                options.os = TargetOs::Wasm;
                options.cpu = TargetCpu::Wasm;
            }
            let mut c = Compiler::new(options, Rc::new(VirtualFs::default()));
            if let Err(e) = c.compile_sources(&[ProgramSource::String(
                include_str!("../../../tests/compiler/semantics/enum-dependencies.jai").into(),
            )]) {
                panic!("{}", e.render(&c.sources));
            }
            let object = dir.join("enums.o");
            emit_object(
                &c.program,
                &jaic_llvm::Options {
                    target: wasm.then(jaic::target::Target::wasm_module),
                    opt_level: level,
                    ..Default::default()
                },
                &object,
            )
            .unwrap();
            if wasm {
                let module = dir.join("enums.wasm");
                run(Command::new("wasm-ld")
                    .args([
                        "-mwasm64",
                        "--no-entry",
                        "--export=enum_dependency_check",
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
                    const result=instance.exports.enum_dependency_check();
                    if(result!==0)throw Error('enum dependency failure '+result);
                "#]).arg(&module));
            } else {
                let driver = dir.join("driver.c");
                std::fs::write(&driver,"#include <stdint.h>\nextern int32_t enum_dependency_check(void); int main(void){return enum_dependency_check();}\n").unwrap();
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
