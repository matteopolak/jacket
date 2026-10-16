//! Private finalized runtime options execute on both real backend targets.
#[path = "support/backend_execution.rs"]
mod backend;

#[test]
fn private_runtime_configuration_executes_native_and_wasm_at_o0_and_o2() {
    backend::execute(
        "runtime-config",
        "configuration_check",
        include_str!("../../../tests/compiler-semantics/runtime-configuration/main.jai"),
        &[],
        Some(65536),
    );
}

#[test]
fn retained_startup_configuration_executes_on_native_and_wasm() {
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
    for wasm in [false, true] {
        for (label, opt_level) in [("O0", OptLevel::O0), ("O2", OptLevel::O2)] {
            let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
                .join(format!("retained-startup-{wasm}-{label}"));
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
                    target: wasm.then(|| "wasm64-unknown-unknown".into()),
                    opt_level,
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
                        "--export=configuration_startup_check",
                    ])
                    .arg(object)
                    .arg("-o")
                    .arg(&module));
                run(Command::new("node").args(["-e",r#"const fs=require('fs'); const m=new WebAssembly.Module(fs.readFileSync(process.argv[1])); if(WebAssembly.Module.imports(m).length)throw Error('unexpected imports'); const x=new WebAssembly.Instance(m,{}); const n=x.exports.configuration_startup_check(); if(n!==0)throw Error('wrong configured size '+n);"#]).arg(module));
            } else {
                let driver = dir.join("driver.c");
                std::fs::write(&driver,"extern int configuration_startup_check(void); int main(void){return configuration_startup_check();}
").unwrap();
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
