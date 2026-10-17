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
    options.preload = Some(library.join(jaic::PRELOAD_ENTRY));
    options.import_paths = vec![library];
    let mut compiler = Compiler::new(options, Rc::new(NativeFs));
    let result = compiler.compile_sources(&[ProgramSource::String(
        "R :: #import \"Runtime_Support\"; main :: () { ctx := R.__jai_runtime_init(0, null); R.__jai_runtime_fini(ctx); }".into(),
    )]);
    (compiler, result)
}

#[test]
fn windows_arm64_entry_backtrace_uses_the_private_native_backend() {
    let (compiler, result) = compile(true, true);
    if let Err(error) = result {
        panic!("{}", error.render(&compiler.sources));
    }
    assert!(compiler.modules.iter().any(|module| {
        module
            .path
            .as_ref()
            .is_some_and(|path| path.ends_with("internal/runtime/windows-arm64-crash/module.jai"))
    }));
    assert!(!compiler.modules.iter().any(|module| {
        module
            .path
            .as_ref()
            .is_some_and(|path| path.ends_with("Runtime_Support_Crash_Handler/module.jai"))
    }));
}

#[test]
fn public_x64_crash_record_facade_still_rejects_windows_arm64() {
    let (mut compiler, result) = compile(false, false);
    result.unwrap();
    let error = compiler
        .compile_sources(&[ProgramSource::String(
            "C :: #import \"Runtime_Support_Crash_Handler\"; main :: () {}".into(),
        )])
        .expect_err("public x64 records cannot represent a native ARM64 context");
    assert!(
        error
            .message
            .contains("the Windows crash handler supports x64 only"),
        "{}",
        error.render(&compiler.sources)
    );
}

#[test]
fn disabled_backtrace_or_embedding_does_not_import_a_native_handler() {
    for (entry, backtrace) in [(true, false), (false, true)] {
        let (compiler, result) = compile(entry, backtrace);
        if let Err(error) = result {
            panic!("{}", error.render(&compiler.sources));
        }
    }
}
