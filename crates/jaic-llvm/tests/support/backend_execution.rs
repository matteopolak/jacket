//! Execute original scalar-result fixtures on native and WASM64, O0/O2.
use jaic::sema::{Compiler, Options, ProgramSource, TargetCpu, TargetOs, VirtualFs};
use jaic_llvm::{OptLevel, emit_object};
use std::{process::Command, rc::Rc};

fn run(command: &mut Command) {
    let output = command.output().expect("required backend test tool");
    assert!(
        output.status.success(),
        "status {:?}: {}",
        output.status.code(),
        String::from_utf8_lossy(&output.stderr)
    );
}

pub fn execute(
    name: &str,
    entry: &str,
    source: &str,
    files: &[(&str, &str)],
    temporary_size: Option<i64>,
) {
    for wasm in [false, true] {
        for optimized in [false, true] {
            let dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(format!(
                "../../target/cleanroom-execution/{name}-{wasm}-{optimized}"
            ));
            std::fs::create_dir_all(&dir).unwrap();
            let mut options = Options::host();
            options.runtime_support = false;
            if let Some(size) = temporary_size {
                options.temporary_storage_size = size;
            }
            if wasm {
                options.os = TargetOs::Wasm;
                options.cpu = TargetCpu::Wasm;
            }
            options.import_paths = vec!["/lib".into()];
            // A fixture may supply its own independent minimal bootstrap declarations.
            if files
                .iter()
                .any(|(path, _)| *path == "/lib/prelude/module.jai")
            {
                options.preload = Some("/lib/prelude/module.jai".into());
            }
            let mut fs = VirtualFs::default();
            for &(path, text) in files {
                fs.insert(path, text.as_bytes());
            }
            let mut c = Compiler::new(options, Rc::new(fs));
            if let Err(error) = c.compile_sources(&[ProgramSource::String(source.into())]) {
                panic!("{}", error.render(&c.sources));
            }
            let object = dir.join("program.o");
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
                let module = dir.join("program.wasm");
                run(Command::new("wasm-ld")
                    .args([
                        "-mwasm64",
                        "--no-entry",
                        "--export-memory",
                        "--allow-undefined",
                    ])
                    .arg(format!("--export={entry}"))
                    .arg(object)
                    .arg("-o")
                    .arg(&module));
                run(Command::new("node").args(["-e",r#"const fs=require('fs');const m=new WebAssembly.Module(fs.readFileSync(process.argv[1]));let memory;const env={};for(const i of WebAssembly.Module.imports(m)){if(i.kind!=='function')throw Error('unexpected import');env[i.name]=()=>{throw Error('unexpected runtime failure '+i.name);};}env.memset=(d,v,n)=>{new Uint8Array(memory.buffer).fill(v,Number(d),Number(d+n));return d;};env.memcpy=(d,s,n)=>{const bytes=new Uint8Array(memory.buffer);bytes.copyWithin(Number(d),Number(s),Number(s+n));return d;};const x=new WebAssembly.Instance(m,{env});memory=x.exports.memory;const result=x.exports[process.argv[2]]();if(result!==0)throw Error('fixture failure '+result);"#]).arg(module).arg(entry));
            } else {
                let driver = dir.join("driver.c");
                std::fs::write(
                    &driver,
                    format!("extern int {entry}(void); int main(void){{return {entry}();}}\n"),
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
