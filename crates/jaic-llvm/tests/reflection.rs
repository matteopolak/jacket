//! Native/WASM execution of independently constructed generic reflection data.
mod common;
use common::run;
use jaic::sema::{Compiler, Options, ProgramSource, TargetCpu, TargetOs, VirtualFs};
use jaic_llvm::{OptLevel, emit_object};
use std::{process::Command, rc::Rc};

#[path = "../../jaic/tests/support/reflection_bootstrap.rs"]
mod bootstrap;

const SOURCE: &str = r#"
Record :: struct #type_info_procedures_are_void_pointers { callback: (int) -> int; n: u8; }
Box :: struct(T: Type = u8, amount: int = size_of(T), label: string = "box") { item: T; }
#program_export "reflection_check"
check :: () -> s32 #c_call {
    a := type_info(Box(u32, 4, "box")); b := type_info(Box(u8, 9, "other"));
    source := a.polymorph_source_struct;
    if source != b.polymorph_source_struct || source.parameters.count != 3 return 1;
    if a.parameters.count != 3 || a.specified_parameters.count != 3 return 2;
    n := a.parameters[1]; original := source.parameters[1];
    if n.type != type_info(int) || n.offset_into_constant_storage < 0 return 3;
    actual := (cast(*int) (a.constant_storage.data + n.offset_into_constant_storage)).*;
    declared := (cast(*int) (source.constant_storage.data + original.offset_into_constant_storage)).*;
    if actual != 4 || declared != 1 return 4;
    t := source.parameters[0];
    default_type := (cast(**Type_Info) (source.constant_storage.data + t.offset_into_constant_storage)).*;
    if default_type != type_info(u8) return 5;
    label := source.parameters[2];
    text := (cast(*string) (source.constant_storage.data + label.offset_into_constant_storage)).*;
    if text != "box" return 6;
    hidden := type_info(Record);
    if hidden.alignment != align_of(Record) return 7;
    if hidden.members[0].type != type_info(*void) || (hidden.members[0].flags & 8)==0 return 8;
    if hidden.members[1].type != type_info(u8) || (hidden.members[1].flags & 8)!=0 return 9;
    return 0;
}
"#;

#[test]
fn generic_default_storage_and_relocations_native_and_wasm() {
    for wasm in [false, true] {
        for optimized in [false, true] {
            let dir = std::path::PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
                .join(format!("reflection-backend/{wasm}-{optimized}"));
            std::fs::create_dir_all(&dir).unwrap();
            let mut options = Options::host();
            options.runtime_support = false;
            options.preload = Some("/lib/prelude/module.jai".into());
            if wasm {
                options.os = TargetOs::Wasm;
                options.cpu = TargetCpu::Wasm;
            }
            let mut fs = VirtualFs::default();
            fs.insert("/lib/prelude/module.jai", bootstrap::BOOTSTRAP.as_bytes());
            let mut compiler = Compiler::new(options, Rc::new(fs));
            if let Err(e) = compiler.compile_sources(&[ProgramSource::String(SOURCE.into())]) {
                panic!("{}", e.render(&compiler.sources));
            }
            let object = dir.join("reflection.o");
            emit_object(
                &compiler.program,
                &jaic_llvm::Options {
                    target: wasm.then(|| jaic::target::Target::wasm_module()),
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
                let module = dir.join("reflection.wasm");
                run(Command::new("wasm-ld")
                    .args([
                        "-mwasm64",
                        "--no-entry",
                        "--export=reflection_check",
                        "--export-memory",
                        "--allow-undefined",
                    ])
                    .arg(&object)
                    .arg("-o")
                    .arg(&module));
                run(Command::new("node").args(["-e", "const fs=require('fs');const m=new WebAssembly.Module(fs.readFileSync(process.argv[1]));let memory;const env={};for(const i of WebAssembly.Module.imports(m)){if(i.kind!=='function')throw Error('unsupported import');env[i.name]=()=>{throw Error('runtime failure '+i.name);};}env.memcmp=(a,b,n)=>{const bytes=new Uint8Array(memory.buffer);a=Number(a);b=Number(b);n=Number(n);for(let i=0;i<n;i++){const diff=bytes[a+i]-bytes[b+i];if(diff)return diff;}return 0;};const x=new WebAssembly.Instance(m,{env});memory=x.exports.memory;const result=x.exports.reflection_check();if(result!==0)throw Error('reflection failure '+result);"]).arg(module));
            } else {
                let driver = dir.join("driver.c");
                std::fs::write(&driver, "extern int reflection_check(void); int main(void) { return reflection_check(); }\n").unwrap();
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
