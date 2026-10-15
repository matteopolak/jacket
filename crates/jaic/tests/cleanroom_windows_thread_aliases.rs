//! Kernel queries through unregistered duplicates must observe real entry completion.
#![cfg(windows)]
use jaic::sema::{Compiler, NativeFs, Options, ProgramSource};
use std::rc::Rc;

#[test]
fn real_kernel_aliases_preserve_exit_codes_and_handle_lifetimes() {
    let mut options = Options::host();
    options.runtime_support = false;
    options.stack_trace = false;
    let mut compiler = Compiler::new(options, Rc::new(NativeFs));
    let source = format!(
        "{}\n#assert check()==0;",
        include_str!("../../../tests/compiler-semantics/windows-thread-alias-lifecycle.jai")
    );
    if let Err(error) = compiler.compile_sources(&[ProgramSource::String(source)]) {
        panic!("{}", error.render(&compiler.sources));
    }
}
