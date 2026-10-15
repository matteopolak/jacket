//! Real Windows thread object semantics for interpreted SDK calls.
#![cfg(windows)]
use jaic::sema::{Compiler, NativeFs, Options, ProgramSource};
use std::rc::Rc;

#[test]
fn managed_handles_names_queries_pseudo_handles_and_errors_are_native() {
    let mut options = Options::host();
    options.runtime_support = false;
    options.stack_trace = false;
    let mut compiler = Compiler::new(options, Rc::new(NativeFs));
    let source = format!(
        "{}\n#assert check() == 0;",
        include_str!("../../../tests/compiler-semantics/windows-thread-description.jai")
    );
    if let Err(error) = compiler.compile_sources(&[ProgramSource::String(source)]) {
        panic!("{}", error.render(&compiler.sources));
    }
}
