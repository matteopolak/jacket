//! `Runtime_Support` on `OS == .NONE`: no libc, no system entry point (docs/native/embedded.md).
use jaic::sema::{Compiler, NativeFs, Options, ProgramSource};
use jaic::target::{TargetCpu, TargetOs};
use std::{path::Path, rc::Rc};

fn compile(source: &str) -> (Compiler, jaic::sema::Result<()>) {
    let library = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../stdlib")
        .canonicalize()
        .unwrap();
    let mut options = Options::host();
    options.os = TargetOs::None;
    options.cpu = TargetCpu::Arm64;
    // What `.AUTO` and an unset `runtime_support_definitions` leave in the options.
    options.runtime_entry_point = true;
    options.runtime_initialization = true;
    options.set_stdlib(&NativeFs, &library);
    options.import_paths = vec![library];
    let mut compiler = Compiler::new(options, Rc::new(NativeFs));
    let result = compiler.compile_sources(&[ProgramSource::String(source.into())]);
    (compiler, result)
}

fn compiled(source: &str) -> Compiler {
    let (compiler, result) = compile(source);
    if let Err(error) = result {
        panic!("{}", error.render(&compiler.src.sources));
    }
    compiler
}

#[test]
fn a_none_program_has_no_system_entry_point_or_libc() {
    let compiler = compiled("#assert OS == .NONE; main :: () {}");
    let program = &compiler.out.program;
    assert!(
        !program
            .funcs
            .iter()
            .flatten()
            .any(|f| f.linkage == jaic::ir::Linkage::Export("main".into())),
        "Runtime_Support defined `main` on a target with no system entry point"
    );
    let libraries: Vec<&str> = program.libraries.iter().map(|l| l.name.as_str()).collect();
    assert!(libraries.is_empty(), "a NONE program links {libraries:?}");
}
