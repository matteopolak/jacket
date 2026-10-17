//! Dependency-order regression, independent of standard-library modules.
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

#[test]
fn nested_record_constants_can_depend_on_earlier_using_fields() {
    let mut options = Options::host();
    options.preload = None;
    options.runtime_support = false;
    let mut compiler = Compiler::new(options, Rc::new(NativeFs));
    let mut facts = IdeFacts::new(vec!["<".into()]);
    facts.lint = true;
    compiler.ide = Some(Box::new(facts));
    compiler.compile_sources(&[ProgramSource::String(
        "Table :: struct { items: [] Padded; Padded :: struct { using info: Payload; INFO_SIZE :: size_of(type_of(info)); PADDING_SIZE :: #run round(INFO_SIZE) - INFO_SIZE; padding: [PADDING_SIZE]u8; } Payload :: struct { owner: *Padded; } }\nround :: (value: s64) -> s64 { return (value + 63) / 64 * 64; }\nmain :: () -> s64 { return Table.Padded.INFO_SIZE; }".into(),
    )]).expect("load nested record declarations");
    let errors = compiler.ide_check_all();
    assert!(
        errors.is_empty(),
        "{:?}",
        errors.iter().map(|e| &e.message).collect::<Vec<_>>()
    );
}

#[test]
fn production_layout_fixture_finishes_strict_ide_checking() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .unwrap();
    let library = root.join("stdlib");
    let source = root.join("tests/platform-io/layout-contract.jai");
    let mut options = Options::host();
    options.preload = Some(library.join(jaic::PRELOAD_ENTRY));
    options.import_paths = vec![library];
    let mut compiler = Compiler::new(options, Rc::new(NativeFs));
    let mut facts = IdeFacts::new(vec![source.to_string_lossy().into_owned()]);
    facts.lint = true;
    compiler.ide = Some(Box::new(facts));
    compiler
        .compile_sources(&[ProgramSource::File(source)])
        .unwrap();
    let errors = compiler.ide_check_all();
    assert!(
        errors.is_empty(),
        "{:?}",
        errors.iter().map(|e| &e.message).collect::<Vec<_>>()
    );
}

#[test]
fn field_type_queries_infer_values_select_branches_and_reject_real_cycles() {
    for query in ["type_of(value)", "type_of(Record.value)"] {
        let mut options = Options::host();
        options.preload = None;
        options.runtime_support = false;
        let mut compiler = Compiler::new(options, Rc::new(NativeFs));
        compiler.ide = Some(Box::new(IdeFacts::new(vec!["<".into()])));
        let source = format!(
            "Record :: struct {{ COUNT :: size_of({query}); #if true {{ value := cast(s64) 1; }} else {{ value: Missing_Type; }} tail: [COUNT]u8; }} main :: () -> s64 {{ return Record.COUNT; }}"
        );
        compiler
            .compile_sources(&[ProgramSource::String(source)])
            .unwrap();
        assert!(compiler.ide_check_all().is_empty());
    }
    let mut options = Options::host();
    options.preload = None;
    options.runtime_support = false;
    let mut compiler = Compiler::new(options, Rc::new(NativeFs));
    let error = compiler.compile_sources(&[ProgramSource::String(
        "Record :: struct { COUNT :: size_of(type_of(value)); value: [COUNT]u8; } main :: () -> s64 { return Record.COUNT; }".into(),
    )]).expect_err("a genuinely recursive type size must reject");
    assert!(
        error.message.contains("cycle in struct field types"),
        "{}",
        error.render(&compiler.sources)
    );
}
