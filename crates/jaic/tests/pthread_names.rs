//! Naming reaches the real SDK only through validated scheduler identities.
#![cfg(any(target_os = "linux", target_os = "macos"))]
use jaic::sema::{Compiler, Options, ProgramSource, VirtualFs};
use std::rc::Rc;

#[test]
fn current_and_managed_worker_names_round_trip_through_the_sdk() {
    let mut options = Options::host();
    options.runtime_support = false;
    let mut compiler = Compiler::new(options, Rc::new(VirtualFs::default()));
    let source = format!(
        "DARWIN :: {};\n{}\n#program_export \"probe\" probe::()->s64 #c_call {{return check();}}",
        cfg!(target_os = "macos"),
        include_str!("../../../tests/compiler-semantics/pthread-names.jai")
    );
    if let Err(error) = compiler.compile_sources(&[ProgramSource::String(source)]) {
        panic!("{}", error.render(&compiler.sources));
    }
    let id = compiler
        .program
        .funcs
        .iter()
        .position(|f| {
            f.as_ref()
                .is_some_and(|f| f.linkage == jaic::ir::Linkage::Export("probe".into()))
        })
        .unwrap();
    let result = compiler
        .interp
        .call(&compiler.program, jaic::ir::FuncId(id as u32), &[])
        .unwrap();
    assert_eq!(result, vec![0], "independent naming contract failure stage");
}
