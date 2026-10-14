//! Reviewed startup options checked against the actual production library.
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
fn windows_arm64_backtrace_has_an_explicit_unsupported_diagnostic() {
    let (compiler, result) = compile(true, true);
    let error = result.expect_err("the reviewed handler has no Windows ARM64 contract");
    assert!(
        error
            .message
            .contains("the reviewed Windows crash handler supports x64 only"),
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
