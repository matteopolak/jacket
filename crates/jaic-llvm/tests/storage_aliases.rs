//! Real C-owned variables shared through filtered reexports, native and WASM64 O0/O2.
mod common;
use common::run;
use jaic::sema::{Compiler, NativeFs, Options, TargetCpu, TargetOs};
use jaic_llvm::{OptLevel, emit_object};
use std::{path::PathBuf, process::Command, rc::Rc};

#[test]
fn filtered_reexports_share_actual_c_memory_native_and_wasm_o0_o2() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let fixture = root.join("tests/compiler-semantics/storage-aliases");
    for wasm in [false, true] {
        for level in [OptLevel::O0, OptLevel::O2] {
            let dir = root.join(format!("target/storage-aliases-backend/{wasm}-{level:?}"));
            std::fs::create_dir_all(&dir).unwrap();
            let mut options = Options::host();
            options.runtime_support = false;
            options.import_paths.push(fixture.clone());
            if wasm {
                options.os = TargetOs::Wasm;
                options.cpu = TargetCpu::Wasm;
            }
            let mut c = Compiler::new(options, Rc::new(NativeFs));
            if let Err(e) = c.compile_program(&fixture.join("main.jai")) {
                panic!("{}", e.render(&c.sources));
            }
            assert_eq!(c.program.foreigns.iter().filter(|f| f.is_data).count(), 2);
            assert!(!c.program.globals.iter().any(|g| {
                ["storage_alias_counter", "storage_alias_pair"].contains(&g.name.as_str())
            }));
            let object = dir.join("aliases.o");
            emit_object(
                &c.program,
                &jaic_llvm::Options {
                    target: wasm.then(|| "wasm64-unknown-unknown".into()),
                    opt_level: level,
                    ..Default::default()
                },
                &object,
            )
            .unwrap();
            if wasm {
                let provider = dir.join("provider.o");
                run(Command::new("clang")
                    .args([
                        "--target=wasm64-unknown-unknown",
                        "-ffreestanding",
                        "-std=c11",
                        "-Wall",
                        "-Wextra",
                        "-Werror",
                        "-c",
                    ])
                    .arg(fixture.join("provider.c"))
                    .arg("-o")
                    .arg(&provider));
                let module = dir.join("aliases.wasm");
                run(Command::new("wasm-ld")
                    .args([
                        "-mwasm64",
                        "--no-entry",
                        "--export=storage_alias_check",
                        "--export-memory",
                        "--allow-undefined",
                    ])
                    .arg(&object)
                    .arg(&provider)
                    .arg("-o")
                    .arg(&module));
                run(Command::new("node").args(["-e",r#"
                    const fs=require('fs');
                    const module=new WebAssembly.Module(fs.readFileSync(process.argv[1]));
                    const env={};let memory;
                    for(const i of WebAssembly.Module.imports(module)) {
                        if(i.kind!=='function')throw Error('unexpected import '+i.name);
                        env[i.name]=()=>{throw Error('unexpected runtime failure '+i.name);};
                    }
                    env.memcpy=(d,s,n)=>{new Uint8Array(memory.buffer).copyWithin(Number(d),Number(s),Number(s+n));return d;};
                    env.memset=(d,v,n)=>{new Uint8Array(memory.buffer).fill(Number(v),Number(d),Number(d+n));return d;};
                    const instance=new WebAssembly.Instance(module,{env});memory=instance.exports.memory;
                    for(let i=0;i<2;i++) {const result=instance.exports.storage_alias_check();if(result!==0)throw Error('storage alias failure '+result);}
                "#]).arg(&module));
            } else {
                let driver = dir.join("driver.c");
                std::fs::write(&driver,"#include <stdint.h>\nextern int32_t storage_alias_check(void); int main(void){for(int i=0;i<2;i++){int32_t r=storage_alias_check();if(r)return r;}return 0;}\n").unwrap();
                let executable = dir.join(format!("driver{}", std::env::consts::EXE_SUFFIX));
                run(Command::new("clang")
                    .args(["-std=c11", "-Wall", "-Wextra", "-Werror"])
                    .arg(&driver)
                    .arg(fixture.join("provider.c"))
                    .arg(&object)
                    .arg("-o")
                    .arg(&executable));
                run(&mut Command::new(executable));
            }
        }
    }
}
