//! Startup options checked against the actual runtime library.
use jaic::sema::{Compiler, NativeFs, Options, ProgramSource, TargetCpu, TargetOs};
use std::{path::Path, rc::Rc};

fn compile(entry: bool, backtrace: bool) -> (Compiler, jaic::sema::Result<()>) {
    let library = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../stdlib")
        .canonicalize()
        .unwrap();
    let mut options = Options::host();
    options.os = TargetOs::Windows;
    options.cpu = TargetCpu::Arm64;
    options.runtime_entry_point = entry;
    options.backtrace_on_crash = backtrace;
    options.set_stdlib(&jaic::sema::NativeFs, &library);
    options.import_paths = vec![library];
    let mut compiler = Compiler::new(options, Rc::new(NativeFs));
    let result = compiler.compile_sources(&[ProgramSource::String(
        "R :: #import \"Runtime_Support\"; main :: () { ctx := R.__jai_runtime_init(0, null); R.__jai_runtime_fini(ctx); }".into(),
    )]);
    (compiler, result)
}

#[test]
fn windows_arm64_entry_backtrace_uses_the_public_native_backend() {
    let (compiler, result) = compile(true, true);
    if let Err(error) = result {
        panic!("{}", error.render(&compiler.src.sources));
    }
    assert!(compiler.ns.modules.iter().any(|module| {
        module
            .path
            .as_ref()
            .is_some_and(|path| path.ends_with("Runtime_Support_Crash_Handler/module.jai"))
    }));
}

#[test]
fn public_crash_record_facade_uses_windows_arm64_layout() {
    let (mut compiler, result) = compile(false, false);
    result.unwrap();
    compiler.compile_sources(&[ProgramSource::String(
        "C :: #import \"Runtime_Support_Crash_Handler\"; #assert size_of(C.CONTEXT) == 912; #assert #exists(C.CONTEXT.Sp); main :: () {}".into(),
    )]).unwrap();
}

#[test]
fn disabled_backtrace_or_embedding_does_not_import_a_native_handler() {
    for (entry, backtrace) in [(true, false), (false, true)] {
        let (compiler, result) = compile(entry, backtrace);
        if let Err(error) = result {
            panic!("{}", error.render(&compiler.src.sources));
        }
        assert!(!compiler.ns.modules.iter().any(|module| {
            module.path.as_ref().is_some_and(|path| {
                path.ends_with("Runtime_Support_Crash_Handler/module.jai")
                    || path.ends_with("internal/runtime/windows-crash-cleanup/module.jai")
            })
        }));
    }
}

#[test]
fn early_host_query_does_not_poison_later_compile_time_context() {
    let library = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../stdlib")
        .canonicalize()
        .unwrap();
    let mut options = Options::host();
    options.set_stdlib(&NativeFs, &library);
    options.import_paths = vec![library];
    let mut compiler = Compiler::new(options, Rc::new(NativeFs));
    let result = compiler.compile_sources(&[ProgramSource::String(
        r#"HOST :: #run,host -> bool { return true; };
#if HOST #run contract();
contract :: () {
    value := sprint("hello %", 42);
    defer free(value);
    assert(value == "hello 42");
}
#import "Basic";
main :: () {}"#
            .into(),
    )]);
    if let Err(error) = result {
        panic!("{}", error.render(&compiler.src.sources));
    }
}
