//! Inherited field layout contracts required by reviewed public record shapes.
use jaic::sema::{Compiler, Options, ProgramSource, VirtualFs};
use std::rc::Rc;

#[test]
fn reflected_struct_alignment_matches_completed_layout() {
    let stdlib = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../stdlib");
    let mut options = Options::host();
    options.import_paths = vec![stdlib.clone()];
    options.preload = Some(stdlib.join(jaic::PRELOAD_ENTRY));
    let mut c = Compiler::new(options, Rc::new(jaic::sema::NativeFs));
    let source = r#"
        Record :: struct { value: u64; }
        Packed :: struct #no_padding { first: u8; second: u64; }
        Wide :: struct #align 32 { value: u8; }
        Box :: struct(T: Type) { value: T; }
        #assert((cast(*Type_Info_Struct) type_info(Record)).alignment == 8);
        #assert((cast(*Type_Info_Struct) type_info(Packed)).alignment == align_of(Packed));
        #assert((cast(*Type_Info_Struct) type_info(Wide)).alignment == 32);
        #assert((cast(*Type_Info_Struct) type_info(Box(u64))).alignment == 8);
        main :: () {}
    "#;
    if let Err(error) = c.compile_sources(&[ProgramSource::String(source.into())]) {
        panic!("{}", error.render(&c.sources));
    }
}

#[test]
fn hidden_procedure_members_keep_their_reflected_marker_and_storage() {
    let stdlib = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../stdlib");
    let mut options = Options::host();
    options.import_paths = vec![stdlib.clone()];
    options.preload = Some(stdlib.join(jaic::PRELOAD_ENTRY));
    let mut c = Compiler::new(options, Rc::new(jaic::sema::NativeFs));
    let source = r#"
        Callback :: #type (value: u32) -> u32 #c_call;
        Hidden :: struct #type_info_procedures_are_void_pointers {
            callback: Callback;
            pointer: *void;
        }
        Plain :: struct { callback: Callback; }
        hidden :: type_info(Hidden);
        plain :: type_info(Plain);
        #assert(hidden.members[0].type == type_info(*void));
        #assert((hidden.members[0].flags & .PROCEDURE_WITH_VOID_POINTER_TYPE_INFO) != 0);
        #assert((hidden.members[0].flags & .CONSTANT) == 0);
        #assert(hidden.members[0].offset_in_bytes == offset_of(Hidden, "callback"));
        #assert(hidden.members[1].type == type_info(*void));
        #assert((hidden.members[1].flags & .PROCEDURE_WITH_VOID_POINTER_TYPE_INFO) == 0);
        #assert(plain.members[0].type == type_info(Callback));
        #assert((plain.members[0].flags & .PROCEDURE_WITH_VOID_POINTER_TYPE_INFO) == 0);
        #assert(type_of(Hidden.callback) == Callback);
        main :: () {}
    "#;
    if let Err(error) = c.compile_sources(&[ProgramSource::String(source.into())]) {
        panic!("{}", error.render(&c.sources));
    }
}

fn compile(source: &str) -> (Compiler, jaic::sema::Result<()>) {
    let mut options = Options::host();
    options.runtime_support = false;
    let mut c = Compiler::new(options, Rc::new(VirtualFs::default()));
    let result = c.compile_sources(&[ProgramSource::String(source.into())]);
    (c, result)
}

#[test]
fn overlay_and_place_resolve_inline_promoted_fields() {
    let (c, result) = compile(
        "Base :: struct { padding: u64; storage: [16] u8; } Middle :: struct { using base: Base; } Outer :: struct { lead: u64; #as using base: Middle; #overlay(storage) value: u64; } Placed :: struct { using base: Base; #place storage; value: u64; } #assert(offset_of(Outer, \"value\") == 16); #assert(offset_of(Outer, \"value\") == offset_of(Outer, \"storage\")); #assert(offset_of(Placed, \"value\") == 8); #assert(size_of(Outer) == 32); object: Outer; #assert(cast(*u8) *object.value == *object.base.base.storage[0]);",
    );
    if let Err(e) = result {
        panic!("{}", e.render(&c.sources));
    }
}

#[test]
fn embedded_using_filter_applies_to_member_lookup() {
    let (c, result) = compile(
        "Base :: struct { x: int; y: int; } Outer :: struct { using,only(x) base: Base; } object: Outer; #assert(*object.x == *object.base.x); #assert(*object.y == *object.base.y);",
    );
    let error = result.expect_err("filtered member must be inaccessible");
    assert!(error.message.contains("y"), "{}", error.render(&c.sources));
}

#[test]
fn overlay_cannot_target_storage_behind_using_pointer() {
    let (c, result) = compile(
        "Base :: struct { storage: [16] u8; } Outer :: struct { using base: *Base; #overlay(storage) value: u64; } #assert(size_of(Outer) > 0);",
    );
    let error = result.expect_err("dynamic pointer target has no inline layout offset");
    assert!(
        error.message.contains("storage"),
        "{}",
        error.render(&c.sources)
    );
}
