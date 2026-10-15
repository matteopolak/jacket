//! Private interpreter transport is optional and cannot relax the public runtime surface.
use jaic::sema::{Compiler, Options, ProgramSource, VirtualFs};
use std::rc::Rc;

fn compiler(hook: &str) -> Compiler {
    let mut fs = VirtualFs::default();
    fs.insert("/lib/prelude/module.jai", b"Source_Code_Location :: struct { fully_pathed_filename: string; line_number: s64; character_number: s64; }\n".as_slice());
    let runtime = format!(
        "#module_parameters(DEFINE_SYSTEM_ENTRY_POINT: bool, DEFINE_INITIALIZATION: bool, ENABLE_BACKTRACE_ON_CRASH: bool);\n#scope_file\n{hook}\n"
    );
    fs.insert("/lib/Runtime_Support/module.jai", runtime.as_bytes());
    let mut options = Options::host();
    options.preload = Some("/lib/prelude/module.jai".into());
    options.import_paths = vec!["/lib".into()];
    Compiler::new(options, Rc::new(fs))
}

#[test]
fn minimal_runtime_does_not_require_the_private_extension() {
    compiler("")
        .compile_sources(&[ProgramSource::String("main :: () {}".into())])
        .unwrap();
}

#[test]
fn malformed_or_compiler_only_handlers_are_rejected() {
    for (hook, message) in [
        ("__jaic_interpreter_assertion :: 7;", "must be a procedure"),
        (
            "__jaic_interpreter_assertion :: () {}",
            "requires (Source_Code_Location, string) -> bool",
        ),
        (
            "__jaic_interpreter_assertion :: (loc: Source_Code_Location, message: string) -> bool #no_context { return false; }",
            "with Jai context",
        ),
        (
            "__jaic_interpreter_assertion :: (loc: Source_Code_Location, message: string) -> bool #compiler { return false; }",
            "ordinary Jai body",
        ),
    ] {
        let mut compiler = compiler(hook);
        let error = compiler
            .compile_sources(&[ProgramSource::String("main :: () {}".into())])
            .expect_err("invalid private transport must diagnose");
        assert!(
            error.message.contains(message),
            "{}",
            error.render(&compiler.sources)
        );
    }
}

#[test]
fn private_transport_is_not_an_application_export() {
    let mut compiler = compiler(
        "handler :: (loc: Source_Code_Location, message: string) -> bool { return false; }\n__jaic_interpreter_assertion :: handler;",
    );
    let error = compiler.compile_sources(&[ProgramSource::String("R :: #import \"Runtime_Support\"; main :: () { R.__jaic_interpreter_assertion(.{}, \"bad\"); }".into())])
        .expect_err("private transport must remain inaccessible");
    assert!(
        error
            .message
            .contains("has no exported member `__jaic_interpreter_assertion`"),
        "{}",
        error.render(&compiler.sources)
    );
}

#[test]
fn interpreter_transport_still_checks_the_native_handler_body() {
    let mut compiler = compiler(
        "handler :: (loc: Source_Code_Location, message: string) -> bool { return missing_value; }\n__jaic_interpreter_assertion :: handler;\n#scope_export\ntrigger :: () { handler(.{}, \"bad\"); }",
    );
    let error = compiler
        .compile_sources(&[ProgramSource::String(
            "#import \"Runtime_Support\"; main :: () { trigger(); }".into(),
        )])
        .expect_err("interpreter transport cannot hide an invalid native body");
    assert!(
        error.message.contains("missing_value"),
        "{}",
        error.render(&compiler.sources)
    );
}
