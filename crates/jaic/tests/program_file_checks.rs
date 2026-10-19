//! Overload clashes that a call must not hide.
use jaic::sema::{Compiler, Options, ProgramSource, VirtualFs};
use std::rc::Rc;

fn compile(source: &str) -> Result<(), String> {
    let mut options = Options::host();
    options.runtime_support = false;
    let mut compiler = Compiler::new(options, Rc::new(VirtualFs::default()));
    compiler
        .compile_sources(&[ProgramSource::String(source.into())])
        .map_err(|error| error.render(&compiler.sources))
}

#[test]
fn identical_overloads_are_rejected_even_when_a_call_names_them() {
    let error = compile("f :: (a: int) {}\nf :: (a: int) {}\nmain :: () { f(1); }\n").unwrap_err();
    assert!(error.contains("identical parameter types"), "{error}");
}
