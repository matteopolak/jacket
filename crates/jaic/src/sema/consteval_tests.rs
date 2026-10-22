//! Compile-time execution is forbidden during contextual type probes.
use super::Compiler;
use crate::sema::{Options, ProgramSource, VirtualFs};
use std::rc::Rc;

#[test]
fn contextual_type_probes_refuse_compile_time_execution() {
    let source = "#run { value: s64; value += 1; } main :: () {}";
    let mut options = Options::host();
    options.runtime_support = false;
    let mut compiler = Compiler::new(options.clone(), Rc::new(VirtualFs::default()));
    compiler.ed.type_probe = true;
    let error = compiler
        .compile_sources(&[ProgramSource::String(source.into())])
        .expect_err("a contextual type probe must not run user code");
    assert_eq!(
        error.message,
        "contextual type inference requires compile-time execution"
    );
    assert_eq!(compiler.ct.executions, 0);
    assert_eq!(compiler.ct.interp.effects, 0);

    // The same program is executable during ordinary compilation.
    let mut compiler = Compiler::new(options, Rc::new(VirtualFs::default()));
    compiler
        .compile_sources(&[ProgramSource::String(source.into())])
        .unwrap();
    assert!(compiler.ct.executions > 0);
}
