//! The native diagnostic extension is private and optional, with strict validation.
use jaic::sema::{Compiler, Options, ProgramSource, VirtualFs};
use std::rc::Rc;

fn compiler(hook: &str) -> Compiler {
    let mut fs = VirtualFs::default();
    let runtime = format!(
        "#module_parameters(DEFINE_SYSTEM_ENTRY_POINT: bool, DEFINE_INITIALIZATION: bool, ENABLE_BACKTRACE_ON_CRASH: bool);\n#scope_file\n{hook}\n"
    );
    fs.insert("/lib/Runtime_Support/module.jai", runtime.as_bytes());
    let mut options = Options::host();
    options.import_paths = vec!["/lib".into()];
    Compiler::new(options, Rc::new(fs))
}

#[test]
fn custom_runtime_does_not_require_an_unreviewed_public_export() {
    let mut compiler = compiler("");
    compiler
        .compile_sources(&[ProgramSource::String("main :: () {}".into())])
        .unwrap();
    assert!(compiler.program.check_failed.is_none());
}

#[test]
fn malformed_private_hook_and_failed_body_are_not_ignored() {
    for (hook, expected) in [
        ("__jaic_check_failed :: 1;", "must be a procedure"),
        ("__jaic_check_failed :: () {}", "must take"),
        (
            "__jaic_check_failed :: (r: s64, a: s64, b: s64, fatal: bool, line: s64, file: *u8) #no_context #no_abc #no_aoc { absent_hook_dependency(); }",
            "absent_hook_dependency",
        ),
    ] {
        let mut compiler = compiler(hook);
        let error = compiler
            .compile_sources(&[ProgramSource::String("main :: () {}".into())])
            .expect_err("invalid hook must fail normally");
        assert!(
            error.message.contains(expected),
            "{}",
            error.render(&compiler.sources)
        );
    }
}
