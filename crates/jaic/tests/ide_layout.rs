//! Original dependency-order regression, independent of standard-library modules.
use jaic::intern::Sym;
use jaic::sema::ide::IdeFacts;
use jaic::sema::{Compiler, EntityId, NativeFs, Options, ProgramSource};
use std::rc::Rc;

#[test]
fn member_constants_see_field_types_before_unused_checking() {
    let mut options = Options::host();
    options.preload = None;
    options.runtime_support = false;
    let mut compiler = Compiler::new(options, Rc::new(NativeFs));
    compiler.ide = Some(Box::new(IdeFacts::new(vec!["<".into()])));
    compiler
        .begin_sources(&[ProgramSource::String(
            "Payload :: struct { value: s64; }\n\
             Record :: struct { info: Payload; INFO_SIZE :: size_of(type_of(info)); padding: [INFO_SIZE]u8; }\n\
             main :: () {}".into(),
        )])
        .expect("load original declarations");
    // Signatures and pointer types can introduce a record before its layout is
    // demanded. Resolve just that type, leaving its member constants unvisited.
    let index = compiler
        .entities
        .iter()
        .position(|entity| entity.name == Sym::intern("Record"))
        .unwrap();
    compiler.resolve_entity(EntityId(index as u32)).unwrap();
    let errors = compiler.ide_check_all();
    assert!(
        errors.is_empty(),
        "{:?}",
        errors.iter().map(|e| &e.message).collect::<Vec<_>>()
    );
}
