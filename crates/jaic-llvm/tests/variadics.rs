//! Real native/WASM execution of mixed variadic argument normalization.
mod common;
use common::run;
use jaic::sema::{Compiler, Options, ProgramSource, TargetCpu, TargetOs, VirtualFs};
use jaic_llvm::{OptLevel, emit_object};
use std::{process::Command, rc::Rc};
#[path = "../../jaic/tests/support/variadic_source.rs"]
mod source;

#[test]
fn mixed_variadics_execute_native_and_wasm_at_o0_and_o2() {
    execute("ordinary", source::SOURCE);
}

fn execute(name: &str, source: &str) {
    for wasm in [false, true] {
        for optimized in [false, true] {
            let dir = std::path::PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
                .join(format!("variadic-backend/{name}-{wasm}-{optimized}"));
            std::fs::create_dir_all(&dir).unwrap();
            let mut options = Options::host();
            options.runtime_support = false;
            if wasm {
                options.os = TargetOs::Wasm;
                options.cpu = TargetCpu::Wasm;
            }
            let mut c = Compiler::new(options, Rc::new(VirtualFs::default()));
            if let Err(e) = c.compile_sources(&[ProgramSource::String(source.into())]) {
                panic!("{}", e.render(&c.sources));
            }
            let object = dir.join("variadics.o");
            emit_object(
                &c.program,
                &jaic_llvm::Options {
                    target: wasm.then(jaic::target::Target::wasm_module),
                    opt_level: if optimized {
                        OptLevel::O2
                    } else {
                        OptLevel::O0
                    },
                    ..Default::default()
                },
                &object,
            )
            .unwrap();
            if wasm {
                let module = dir.join("variadics.wasm");
                run(Command::new("wasm-ld")
                    .args([
                        "-mwasm64",
                        "--no-entry",
                        "--export=variadic_check",
                        "--export-memory",
                        "--allow-undefined",
                    ])
                    .arg(object)
                    .arg("-o")
                    .arg(&module));
                run(Command::new("node").args(["-e",r#"const fs=require('fs');const m=new WebAssembly.Module(fs.readFileSync(process.argv[1]));let memory;const env={};for(const i of WebAssembly.Module.imports(m)){if(i.kind!=='function')throw Error('unexpected import');env[i.name]=()=>{throw Error('unexpected runtime failure '+i.name);};}env.memcpy=(d,s,n)=>{const bytes=new Uint8Array(memory.buffer);bytes.copyWithin(Number(d),Number(s),Number(s+n));return d;};const x=new WebAssembly.Instance(m,{env});memory=x.exports.memory;const result=x.exports.variadic_check();if(result!==0)throw Error('variadic failure '+result);"#]).arg(module));
            } else {
                let driver = dir.join("driver.c");
                std::fs::write(
                    &driver,
                    "extern int variadic_check(void); int main(void){return variadic_check();}\n",
                )
                .unwrap();
                let exe = dir.join("driver");
                run(Command::new("clang")
                    .arg(driver)
                    .arg(object)
                    .arg("-o")
                    .arg(&exe));
                run(&mut Command::new(exe));
            }
        }
    }
}

#[test]
fn dynamic_packs_survive_continuation_unwind_and_rewind() {
    execute(
        "continuation",
        r#"
    Control :: struct { state: u64; top: *u8; limit: *u8; enabled: u64; }
    control: Control;
    frames: [8192]u8;
    tail: [2]int = .[2,3];
    #program_export "jaic_wasi_green_control"
    get_control :: () -> *Control #no_context { return *control; }
    #program_export "jaic_wasi_frames_exhausted"
    exhausted :: () #no_context { #asm { int3; } }
    #program_export "pthread_create"
    spawn :: () #no_context {}
    #program_export "jaic_wasi_suspend"
    suspend :: () #no_context {
        if control.state == 2 { control.state=0; return; }
        control.state=1;
    }
    sum :: (items: ..int) -> int #no_context {
        suspend();
        result:int; for items result=result*10+it;
        return result;
    }
    thread :: () -> int #no_context { return sum(1,..tail,4); }
    #program_export "jaic_wasi_run_threads"
    schedule :: () -> s32 #no_context {
        spawn();
        for 0..3 {
            control.top=frames.data; control.limit=frames.data+frames.count;
            control.state=0;
            result:=thread();
            if control.state!=1 return 1;
            // Clobber unrelated native-stack storage between the two walks.
            noise:[1024]u8; for *noise it.*=cast(u8)(it_index & 255);
            control.top=frames.data; control.state=2;
            result=thread();
            if control.state!=0 || result!=1234 return 2;
            if control.top!=frames.data return 3;
        }
        return 0;
    }
    #program_export "variadic_check"
    check :: () -> s32 #c_call { return schedule(); }
    "#,
    );
}
