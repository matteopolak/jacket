//! Private finalized runtime options execute on both real backend targets.
use jaic::sema::{Compiler, ProgramSource, VirtualFs};
use std::{process::Command, rc::Rc};
use test_support::backend::{self, VARIANTS};
use test_support::{TempDir, process};

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
    if !backend::tools_available() {
        return;
    }
    for variant in VARIANTS {
        let dir = TempDir::new(&format!("{name}-{}", variant.label()));
        let mut fs = VirtualFs::default();
        fs.insert("/lib/Runtime_Support/module.jai", runtime.as_bytes());
        let mut options = variant.options();
        options.runtime_support = true;
        options.import_paths = vec!["/lib".into()];
        options.temporary_storage_size = 1_048_576;
        let mut c = Compiler::new(options, Rc::new(fs));
        c.compile_sources(&[ProgramSource::String("main::(){}".into())])
            .map_err(|e| e.render(&c.src.sources))
            .unwrap();
        c.prepare_compiled_output()
            .map_err(|e| e.render(&c.src.sources))
            .unwrap();
        let object = dir.join("program.o");
        variant.emit(&c, &object, None);
        if variant.wasm {
            let module = dir.join("program.wasm");
            let exports: &[&str] = if inspect_storage {
                &["configuration_startup_check", "storage_address"]
            } else {
                &["configuration_startup_check"]
            };
            backend::link_wasm(&[&object], &module, exports);
            backend::node_eval(STARTUP_HOST, &[&module]);
        } else {
            let driver = if inspect_storage {
                "extern int configuration_startup_check(void); extern unsigned char *storage_address(void); int main(void){int n=configuration_startup_check(); if(n)return n; unsigned char *p=storage_address();return p[0]!=19||p[1048575]!=73;}\n"
            } else {
                "extern int configuration_startup_check(void); int main(void){return configuration_startup_check();}\n"
            };
            let exe = backend::link_native(&dir, driver, &[&object]);
            process::run(&mut Command::new(exe));
        }
    }
}

/// A wasm host without imports: the startup check must return 0, and when the module exports
/// `storage_address` the configured storage must hold the bytes the check wrote.
const STARTUP_HOST: &str = r"
const fs=require('fs');
const m=new WebAssembly.Module(fs.readFileSync(process.argv[1]));
if(WebAssembly.Module.imports(m).length)throw Error('unexpected imports');
const x=new WebAssembly.Instance(m,{});
const n=x.exports.configuration_startup_check();
if(n!==0)throw Error('wrong configured size '+n);
if(x.exports.storage_address){
  const p=Number(x.exports.storage_address());
  const b=new Uint8Array(x.exports.memory.buffer);
  if(b[p]!==19||b[p+1048575]!==73)throw Error('wrong actual storage');
}
";

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
