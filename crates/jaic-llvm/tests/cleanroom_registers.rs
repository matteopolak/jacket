//! Real native/WASM execution of mixed register operand forwarding.
use jaic::sema::{Compiler, Options, ProgramSource, TargetCpu, TargetOs, VirtualFs};
use jaic_llvm::{OptLevel, emit_object};
use std::{process::Command, rc::Rc};
#[path = "../../jaic/tests/support/register_source.rs"]
mod source;

fn run(command: &mut Command) {
    let output = command.output().expect("required backend test tool");
    assert!(
        output.status.success(),
        "status {:?}: {}",
        output.status.code(),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn register_operands_execute_native_and_wasm_at_o0_and_o2() {
    execute("registers", source::SOURCE);
}

fn execute(name: &str, source: &str) {
    for wasm in [false, true] {
        for optimized in [false, true] {
            let dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(format!(
                "../../target/cleanroom-register-backend/{name}-{wasm}-{optimized}"
            ));
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
                    target: wasm.then(|| "wasm64-unknown-unknown".into()),
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
                        "--export=register_check",
                        "--export-memory",
                        "--allow-undefined",
                    ])
                    .arg(object)
                    .arg("-o")
                    .arg(&module));
                run(Command::new("node").args(["-e",r#"const fs=require('fs');const m=new WebAssembly.Module(fs.readFileSync(process.argv[1]));let memory;const env={};for(const i of WebAssembly.Module.imports(m)){if(i.kind!=='function')throw Error('unexpected import');env[i.name]=()=>{throw Error('unexpected runtime failure '+i.name);};}env.memset=(d,v,n)=>{new Uint8Array(memory.buffer).fill(v,Number(d),Number(d+n));return d;};env.memcpy=(d,s,n)=>{const bytes=new Uint8Array(memory.buffer);bytes.copyWithin(Number(d),Number(s),Number(s+n));return d;};const x=new WebAssembly.Instance(m,{env});memory=x.exports.memory;const result=x.exports.register_check();if(result!==0)throw Error('variadic failure '+result);"#]).arg(module));
            } else {
                let driver = dir.join("driver.c");
                std::fs::write(
                    &driver,
                    "extern int register_check(void); int main(void){return register_check();}\n",
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
