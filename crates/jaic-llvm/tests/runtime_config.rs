//! Private finalized runtime options execute on both real backend targets.
#[path = "support/backend_execution.rs"]
mod backend;
mod common;

#[test]
fn private_runtime_configuration_executes_native_and_wasm_at_o0_and_o2() {
    backend::execute(
        "runtime-config",
        "configuration_check",
        include_str!("../../../tests/compiler/semantics/runtime-configuration/main.jai"),
        &[],
        Some(65536),
    );
}

fn execute_startup_runtime(name: &str, runtime: &str, inspect_storage: bool) {
    use jaic::sema::{Compiler, Options, ProgramSource, TargetCpu, TargetOs, VirtualFs};
    use jaic_llvm::{OptLevel, emit_object};
    use std::{path::PathBuf, process::Command, rc::Rc};
    fn run(command: &mut Command) {
        let out = command
            .output()
            .expect("required source-built backend tool");
        assert!(
            out.status.success(),
            "{command:?}: {}",
            String::from_utf8_lossy(&out.stderr)
        );
    }
    for wasm in [false, true] {
        for (label, opt_level) in [("O0", OptLevel::O0), ("O2", OptLevel::O2)] {
            let dir =
                PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(format!("{name}-{wasm}-{label}"));
            std::fs::create_dir_all(&dir).unwrap();
            let mut fs = VirtualFs::default();
            fs.insert("/lib/Runtime_Support/module.jai", runtime.as_bytes());
            let mut options = Options::host();
            options.import_paths = vec!["/lib".into()];
            options.temporary_storage_size = 1048576;
            if wasm {
                options.os = TargetOs::Wasm;
                options.cpu = TargetCpu::Wasm;
            }
            let mut c = Compiler::new(options, Rc::new(fs));
            c.compile_sources(&[ProgramSource::String("main::(){}".into())])
                .map_err(|e| e.render(&c.sources))
                .unwrap();
            c.prepare_compiled_output()
                .map_err(|e| e.render(&c.sources))
                .unwrap();
            let object = dir.join("program.o");
            emit_object(
                &c.program,
                &jaic_llvm::Options {
                    target: wasm.then(jaic::target::Target::wasm_module),
                    opt_level,
                    ..Default::default()
                },
                &object,
            )
            .unwrap();
            if wasm {
                let module = dir.join("program.wasm");
                let mut linker = Command::new("wasm-ld");
                if inspect_storage {
                    linker.arg("--export=storage_address");
                }
                run(linker
                    .args([
                        "-mwasm64",
                        "--no-entry",
                        "--export-memory",
                        "--export=configuration_startup_check",
                    ])
                    .arg(object)
                    .arg("-o")
                    .arg(&module));
                run(Command::new("node").args(["-e",r#"const fs=require('fs'); const m=new WebAssembly.Module(fs.readFileSync(process.argv[1])); if(WebAssembly.Module.imports(m).length)throw Error('unexpected imports'); const x=new WebAssembly.Instance(m,{}); const n=x.exports.configuration_startup_check(); if(n!==0)throw Error('wrong configured size '+n); if(x.exports.storage_address){const p=Number(x.exports.storage_address());const b=new Uint8Array(x.exports.memory.buffer);if(b[p]!==19||b[p+1048575]!==73)throw Error('wrong actual storage');}"#]).arg(module));
            } else {
                let driver = dir.join("driver.c");
                std::fs::write(&driver,if inspect_storage {
                        "extern int configuration_startup_check(void); extern unsigned char *storage_address(void); int main(void){int n=configuration_startup_check(); if(n)return n; unsigned char *p=storage_address();return p[0]!=19||p[1048575]!=73;}\n"
                    } else {
                        "extern int configuration_startup_check(void); int main(void){return configuration_startup_check();}\n"
                    }).unwrap();
                let executable = dir.join("driver");
                run(Command::new("clang")
                    .arg(driver)
                    .arg(object)
                    .arg("-o")
                    .arg(&executable));
                run(&mut Command::new(executable));
            }
        }
    }
}

#[test]
fn retained_startup_configuration_executes_on_native_and_wasm() {
    let runtime = r#"
        #module_parameters(DEFINE_SYSTEM_ENTRY_POINT:bool, DEFINE_INITIALIZATION:bool, ENABLE_BACKTRACE_ON_CRASH:bool);
        #scope_file
        __jaic_runtime_temporary_storage_size :: () -> s64 #intrinsic #no_context;
        read_size :: () -> s64 #no_context {return __jaic_runtime_temporary_storage_size();}
        #no_reset getter: () -> s64 #no_context;
        #run {getter=read_size;}
        #program_export "configuration_startup_check"
        startup :: () -> s32 #c_call {return ifx getter()==1048576 then 0 else 1;}
    "#;
    execute_startup_runtime("retained-startup", runtime, false);
}

#[test]
fn public_constant_storage_executes_on_native_and_wasm() {
    execute_startup_runtime(
        "public-constant-startup",
        r#"
        #module_parameters(DEFINE_SYSTEM_ENTRY_POINT:bool, DEFINE_INITIALIZATION:bool, ENABLE_BACKTRACE_ON_CRASH:bool);
        #scope_file
        Bytes::TEMPORARY_STORAGE_SIZE;
        Buffer::struct {bytes:[Bytes]u8;}
        #assert size_of(Buffer)==1048576;
        storage:Buffer;
        #program_export "storage_address"
        address::()->*u8 #c_call {return storage.bytes.data;}
        #program_export "configuration_startup_check"
        startup::()->s32 #c_call {
            if storage.bytes.count!=1048576 return 1;
            storage.bytes[0]=19;
            storage.bytes[1048575]=73;
            return 0;
        }
    "#,
        true,
    );
}
