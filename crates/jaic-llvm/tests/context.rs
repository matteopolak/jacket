//! Execute original context/cleanup control-flow cases through both backends.
mod common;
use common::run;
use jaic::sema::{Compiler, Options, ProgramSource, TargetCpu, TargetOs, VirtualFs};
use jaic_llvm::{OptLevel, emit_object};
use std::{process::Command, rc::Rc};

const SOURCE: &str = r#"
#add_context marker: s64 = 7;
observed: int;
conditional :: (enabled: bool) #expand {
    if enabled { { copied := context; `push_context,defer_pop copied; `defer observed += context.marker; } }
}
run :: (enabled: bool) -> bool {
    context.marker = 55;
    { conditional(enabled); context.marker = 91; }
    return context.marker == (ifx enabled then 55 else 91);
}
early :: () -> int {
    context.marker = 55;
    defer observed = context.marker;
    push_context { context.marker = 123; return context.marker; }
}
#program_export "context_check"
check :: () -> s32 #c_call {
    push_context {
        if !run(false) || observed != 0 return 1;
        if !run(true) || observed != 91 return 2;
        if early() != 123 || observed != 55 return 3;
        for i: 0..3 {
            context.marker = 55;
            conditional(true);
            context.marker = 91;
            if i < 2 continue;
            break;
        }
        if context.marker != 55 return 4;
    }
    return 0;
}
"#;

#[test]
fn conditional_context_and_exit_cleanup_native_and_wasm() {
    for wasm in [false, true] {
        for optimized in [false, true] {
            let dir = std::path::PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
                .join(format!("context-backend/{wasm}-{optimized}"));
            std::fs::create_dir_all(&dir).unwrap();
            let mut options = Options::host();
            options.runtime_support = false;
            if wasm {
                options.os = TargetOs::Wasm;
                options.cpu = TargetCpu::Wasm;
            }
            let mut compiler = Compiler::new(options, Rc::new(VirtualFs::default()));
            if let Err(e) = compiler.compile_sources(&[ProgramSource::String(SOURCE.into())]) {
                panic!("{}", e.render(&compiler.sources));
            }
            let object = dir.join("context.o");
            emit_object(
                &compiler.program,
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
                let module = dir.join("context.wasm");
                run(Command::new("wasm-ld")
                    .args([
                        "-mwasm64",
                        "--no-entry",
                        "--export=context_check",
                        "--allow-undefined",
                    ])
                    .arg(&object)
                    .arg("-o")
                    .arg(&module));
                run(Command::new("node").args(["-e", "const fs=require('fs');const m=new WebAssembly.Module(fs.readFileSync(process.argv[1]));const env={};for(const i of WebAssembly.Module.imports(m)){if(i.kind!=='function')throw Error('unsupported import');env[i.name]=()=>{throw Error('runtime failure '+i.name);};}const x=new WebAssembly.Instance(m,{env});const result=x.exports.context_check();if(result!==0)throw Error('context failure '+result);"]).arg(module));
            } else {
                let driver = dir.join("driver.c");
                std::fs::write(
                    &driver,
                    "extern int context_check(void); int main(void) { return context_check(); }\n",
                )
                .unwrap();
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
