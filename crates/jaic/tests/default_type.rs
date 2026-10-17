//! Default-value identity checks, without calling a Type formatter.
use jaic::{
    sema::{
        Compiler, Options, VirtualFs,
        value::{Aggregate, Value},
    },
    source::Span,
    types::TypeId,
};
use std::{path::Path, rc::Rc};

#[path = "support/reflection_bootstrap.rs"]
mod bootstrap;

fn compiler(source: &str) -> Compiler {
    let mut fs = VirtualFs::default();
    fs.insert("/lib/prelude/module.jai", bootstrap::BOOTSTRAP.as_bytes());
    fs.insert("/app/main.jai", source.as_bytes());
    let mut options = Options::host();
    options.runtime_support = false;
    options.preload = Some("/lib/prelude/module.jai".into());
    let mut compiler = Compiler::new(options, Rc::new(fs));
    if let Err(error) = compiler.compile_program(Path::new("/app/main.jai")) {
        panic!("{}", error.render(&compiler.sources));
    }
    compiler
}

#[test]
fn type_default_image_has_the_same_relocation_as_explicit_void() {
    let mut compiler = compiler("");
    let image = compiler
        .default_initializer(TypeId::TYPE, Span::NONE)
        .unwrap()
        .expect("Type requires a descriptor, not zero bytes");
    let mut explicit = Aggregate::zeroed(8, Span::NONE).unwrap();
    compiler
        .write_value(
            &mut explicit,
            0,
            &Value::Type(TypeId::VOID),
            TypeId::TYPE,
            Span::NONE,
        )
        .unwrap();
    assert_eq!(image.bytes, explicit.bytes);
    assert_eq!(image.relocs, explicit.relocs);
    assert_eq!(image.relocs.len(), 1);
    let pointer = compiler.types.pointer(TypeId::VOID);
    assert!(
        compiler
            .default_initializer(pointer, Span::NONE)
            .unwrap()
            .is_none()
    );
}

fn evaluates_true(body: &str) {
    let source = format!("#program_export \"probe\" probe :: () -> bool #c_call {{ {body} }}");
    evaluates_program(&source);
}

fn evaluates_program(source: &str) {
    let mut compiler = compiler(source);
    let id = compiler
        .program
        .funcs
        .iter()
        .position(|f| {
            f.as_ref()
                .is_some_and(|f| f.linkage == jaic::ir::Linkage::Export("probe".into()))
        })
        .unwrap();
    assert_eq!(
        compiler
            .interp
            .call(&compiler.program, jaic::ir::FuncId(id as u32), &[])
            .unwrap(),
        vec![1]
    );
}

#[test]
fn scalar_and_nested_record_defaults_are_void() {
    evaluates_true(
        "Record :: struct { tag:Type; count:int; } Outer :: struct { pad:u8; child:Record; } scalar:Type; record:Outer; return scalar==void && record.child.tag==void && record.child.count==0 && record.pad==0;",
    );
}

#[test]
fn fixed_and_nested_arrays_preserve_descriptor_relocations() {
    evaluates_true(
        "Record :: struct { tag:Type; } values:[2][2]Record; return values[0][0].tag==void && values[0][1].tag==void && values[1][0].tag==void && values[1][1].tag==void;",
    );
}

#[test]
fn generic_record_default_uses_its_type_argument() {
    evaluates_true("Box :: struct(T:Type) { item:T; } value:Box(Type); return value.item==void;");
}

#[test]
fn distinct_record_default_preserves_the_underlying_image() {
    evaluates_true(
        "Record :: struct { tag:Type; } Distinct :: #type,distinct Record; value:Distinct; ordinary:=cast(Record)value; return ordinary.tag==void;",
    );
}

#[test]
fn explicit_nonvoid_fields_override_default_images() {
    evaluates_true(
        "Record :: struct { implicit:Type; explicit:Type=u8; } value:Record; changed:Record=.{explicit=s64}; return value.implicit==void && value.explicit==u8 && changed.implicit==void && changed.explicit==s64;",
    );
}

#[test]
fn global_scalar_and_record_defaults_use_the_descriptor_image() {
    evaluates_program(
        "global:Type; Record :: struct { tag:Type; count:int; } record:Record; #program_export \"probe\" probe::()->bool #c_call { return global==void && record.tag==void && record.count==0; }",
    );
}

#[test]
fn generated_record_initializer_installs_the_default_descriptor() {
    evaluates_true(
        "Record :: struct { tag:Type; count:int; } value:Record=---; info:=type_info(Record); info.initializer(cast(*void)*value); return value.tag==void && value.count==0;",
    );
}
