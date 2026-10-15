//! Original generic-default metadata regressions using reviewed record fields.
use jaic::sema::{Compiler, Options, VirtualFs};
use std::{path::Path, rc::Rc};

#[path = "support/reflection_bootstrap.rs"]
mod bootstrap;
use bootstrap::BOOTSTRAP;

#[test]
fn hidden_procedure_members_keep_the_marker_and_struct_alignment() {
    check(
        r#"
        Record :: struct #type_info_procedures_are_void_pointers { callback: (int) -> int; n: u8; }
        info :: type_info(Record);
        #assert(info.alignment == align_of(Record));
        #assert(info.members.count == 2);
        #assert(info.members[0].type == type_info(*void));
        #assert((info.members[0].flags & 8) != 0);
        #assert((info.members[1].flags & 8) == 0);
        #assert(info.members[1].type == type_info(u8));
        #assert(info.members[1].offset_in_bytes == 8);
    "#,
    );
}

#[test]
fn forward_overlay_reflection_preserves_declaration_order_offsets_and_flags() {
    check(
        r#"
        R :: struct { #overlay(anchor) alias:u32; tag:u32; anchor:u64; }
        info :: type_info(R);
        #assert info.members.count==3 && info.alignment==8 && info.runtime_size==16;
        #assert info.members[0].name=="alias" && info.members[0].offset_in_bytes==8;
        #assert info.members[1].name=="tag" && info.members[1].offset_in_bytes==0;
        #assert info.members[2].name=="anchor" && info.members[2].offset_in_bytes==8;
        #assert (info.members[0].flags & 0x20)!=0;
        #assert (info.members[1].flags & 0x20)==0 && (info.members[2].flags & 0x20)==0;
    "#,
    );
}

fn check(source: &str) {
    let (c, result) = compile(source);
    if let Err(e) = result {
        panic!("{}", e.render(&c.sources));
    }
}

fn compile(source: &str) -> (Compiler, jaic::sema::Result<()>) {
    let mut fs = VirtualFs::default();
    fs.insert("/lib/prelude/module.jai", BOOTSTRAP.as_bytes());
    fs.insert("/app/main.jai", source.as_bytes());
    let mut options = Options::host();
    options.runtime_support = false;
    options.preload = Some("/lib/prelude/module.jai".into());
    let mut c = Compiler::new(options, Rc::new(fs));
    let result = c.compile_program(Path::new("/app/main.jai"));
    (c, result)
}

#[test]
fn adding_context_after_layout_remains_an_error() {
    let (mut compiler, result) = compile("#run size_of(#Context);");
    result.unwrap();
    let error = compiler
        .add_source(&jaic::sema::ProgramSource::String(
            "#add_context late_field:bool;".into(),
        ))
        .expect_err("a finalized Context must not silently change layout");
    assert!(
        error
            .message
            .contains("#add_context after the Context type was laid out")
    );
}

#[test]
fn generic_source_defaults_have_types_values_and_shared_identity() {
    check(
        r#"
Box :: struct(T: Type = int, amount: int = 3) { value: T; }
a :: type_info(Box(int, 3)); b :: type_info(Box(u8, 9));
#assert(a.specified_parameters.count == 2);
#assert(a.polymorph_source_struct == b.polymorph_source_struct);
#assert(a.polymorph_source_struct.parameters.count == 2);
#assert(a.polymorph_source_struct.parameters[0].name == "T");
#assert(a.polymorph_source_struct.parameters[1].name == "amount");
check :: () -> bool {
    source := a.polymorph_source_struct;
    t := source.parameters[0]; n := source.parameters[1];
    if t.type != type_info(Type) || n.type != type_info(int) return false;
    if t.offset_into_constant_storage < 0 || n.offset_into_constant_storage < 0 return false;
    actual_type := (cast(**Type_Info) (source.constant_storage.data + t.offset_into_constant_storage)).*;
    actual_n := (cast(*int) (source.constant_storage.data + n.offset_into_constant_storage)).*;
    return actual_type == type_info(int) && actual_n == 3;
}
#assert(check());
"#,
    );
}

#[test]
fn parameters_without_defaults_retain_negative_storage_offsets() {
    check(
        r#"
Box :: struct(T: Type, amount: int = 3) { value: T; }
info :: type_info(Box(u8));
source :: info.polymorph_source_struct;
#assert(source == type_info(Box));
#assert(source.parameters.count == 2);
#assert(source.parameters[0].offset_into_constant_storage == -1);
#assert(source.parameters[1].offset_into_constant_storage >= 0);
#assert(info.specified_parameters.count == 2);
"#,
    );
}

#[test]
fn instance_defaults_evaluate_against_preceding_actual_bindings() {
    check(
        r#"
Box :: struct(T: Type = u8, amount: int = size_of(T)) { value: T; }
info :: type_info(Box(u32, 4));
check :: () -> bool {
    if info.parameters.count != 2 return false;
    n := info.parameters[1];
    if n.offset_into_constant_storage < 0 return false;
    actual := (cast(*int) (info.constant_storage.data + n.offset_into_constant_storage)).*;
    source := info.polymorph_source_struct;
    original := source.parameters[1];
    declared := (cast(*int) (source.constant_storage.data + original.offset_into_constant_storage)).*;
    return actual == 4 && declared == 1;
}

#assert(check());
"#,
    );
}

#[test]
fn dependent_defaults_are_concrete_on_instances_and_symbolic_on_sources() {
    check(
        r#"
Box :: struct(T: Type, amount: int = size_of(T), value: T = 0) { item: T; }
info :: type_info(Box(u32));
check :: () -> bool {
    if info.parameters.count != 3 return false;
    n := info.parameters[1]; v := info.parameters[2];
    if n.type != type_info(int) || v.type != type_info(u32) return false;
    actual := (cast(*int) (info.constant_storage.data + n.offset_into_constant_storage)).*;
    zero := (cast(*u32) (info.constant_storage.data + v.offset_into_constant_storage)).*;
    source := info.polymorph_source_struct;
    return actual == 4 && zero == 0 && source.parameters.count == 3
        && source.parameters[1].offset_into_constant_storage == -1
        && source.parameters[2].type == null;
}
#assert(check());
"#,
    );
}

#[test]
fn inferred_pattern_types_are_reflected_with_concrete_instance_bindings() {
    check(
        r"Box :: struct(value: $T, amount: int = size_of(T)) { item: T; } info :: type_info(Box(cast(u32) 8)); #assert(info.parameters.count == 2); #assert(info.parameters[0].type == type_info(u32)); #assert(info.parameters[0].offset_into_constant_storage == -1); #assert(info.parameters[1].offset_into_constant_storage >= 0); #assert(info.polymorph_source_struct.parameters[0].type == null);",
    );
}

#[test]
fn invalid_defaults_are_errors_even_when_explicit_arguments_bypass_them() {
    let (c, result) = compile(
        r"Box :: struct(T: Type = u8, amount: int = missing_default_symbol) { item: T; } info :: type_info(Box(u8, 4));",
    );
    let error = result.expect_err("reflection cannot discard invalid defaults");
    assert!(
        error.message.contains("missing_default_symbol"),
        "{}",
        error.render(&c.sources)
    );
}
