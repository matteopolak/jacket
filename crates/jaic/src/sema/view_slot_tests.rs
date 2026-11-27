//! The stack slots lowering makes for strings and views take the target's view layout.
use crate::sema::{Compiler, Options, ProgramSource, VirtualFs};
use crate::target::{Target, TargetCpu, TargetOs};
use std::rc::Rc;

/// On i686 a view is a 4-aligned `s64` count then a 4-byte pointer: 12 bytes, not the 16 of a
/// 64-bit target. A string literal and an empty variadic argument both get such a slot.
#[test]
fn a_string_literal_and_an_empty_variadic_get_a_slot_of_the_targets_view_size() {
    let source = "
        takes :: (s: string) {}
        many :: (values: ..int) {}
        caller :: () { takes(\"hi\"); many(); }
        main :: () { caller(); }
    ";
    let mut options = Options::host();
    options.runtime_support = false;
    Target::new(TargetOs::Linux, TargetCpu::X86).apply(&mut options);
    let view = options.layout.builtins().view;
    assert_eq!((view.size, view.align), (12, 4));
    let mut compiler = Compiler::new(options, Rc::new(VirtualFs::default()));
    if let Err(error) = compiler.compile_sources(&[ProgramSource::String(source.into())]) {
        panic!("{}", error.render(&compiler.src.sources));
    }
    let program = &*compiler.out.program;
    let caller = program
        .funcs
        .iter()
        .flatten()
        .find(|f| f.name == "caller")
        .expect("caller is lowered");
    let slots: Vec<(u64, u64)> = caller.slots.iter().map(|s| (s.size, s.align)).collect();
    assert!(
        slots.contains(&(12, 4)) && !slots.contains(&(16, 8)),
        "the views of `caller` are 12 bytes: {slots:?}"
    );
}
