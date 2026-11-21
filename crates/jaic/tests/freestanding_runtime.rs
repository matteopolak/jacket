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

/// Compiles `source` for the target and prepares the output, as a build does.
fn prepared(source: &str) -> Result<(), String> {
    let (mut compiler, result) = compile(source);
    result.map_err(|e| e.render(&compiler.src.sources))?;
    compiler
        .prepare_compiled_output()
        .map_err(|e| e.render(&compiler.src.sources))
}

#[test]
fn a_reachable_c_library_procedure_is_a_compile_error_naming_it() {
    let error = prepared(
        r#"
        libc :: #system_library "libc";
        puts :: (text: *u8) -> s32 #foreign libc;
        #program_export "start"
        start :: () #c_call #no_context { puts("hi"); }
        main :: () {}
        "#,
    )
    .unwrap_err();
    assert!(error.contains("`puts`"), "{error}");
    assert!(error.contains("libc"), "{error}");
    assert!(error.contains("freestanding"), "{error}");
}

#[test]
fn an_unreachable_c_library_procedure_is_not_an_error() {
    prepared(
        r#"
        libc :: #system_library "libc";
        puts :: (text: *u8) -> s32 #foreign libc;
        main :: () {}
        "#,
    )
    .unwrap();
}

#[test]
fn a_symbol_bound_with_elsewhere_is_left_to_the_linker() {
    prepared(
        r#"
        board_init :: () #elsewhere #c_call;
        #program_export "start"
        start :: () #c_call #no_context { board_init(); }
        main :: () {}
        "#,
    )
    .unwrap();
}

#[test]
fn the_runtime_defines_the_c_memory_functions_the_compiler_calls() {
    // No `main`: the program is its own entry point, and the runtime's exports still count.
    let compiler = compiled(r#"#program_export "start" start :: () #c_call #no_context {}"#);
    let exported: Vec<String> = compiler
        .out
        .program
        .funcs
        .iter()
        .flatten()
        .filter_map(|f| match &f.linkage {
            jaic::ir::Linkage::Export(name) => Some(name.clone()),
            jaic::ir::Linkage::Internal => None,
        })
        .collect();
    for symbol in ["memcpy", "memmove", "memset", "memcmp", "bcmp", "strlen"] {
        assert!(
            exported.iter().any(|n| n == symbol),
            "{symbol} in {exported:?}"
        );
    }
}
