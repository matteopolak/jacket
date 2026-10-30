//! Native/WASM execution of independently constructed generic reflection data.
use test_support::backend;

#[path = "../../jaic/tests/support/reflection_bootstrap.rs"]
mod bootstrap;

const SOURCE: &str = r#"
Record :: struct #type_info_procedures_are_void_pointers { callback: (int) -> int; n: u8; }
Box :: struct(T: Type = u8, amount: int = size_of(T), label: string = "box") { item: T; }
#program_export "reflection_check"
check :: () -> s32 #c_call {
    a := type_info(Box(u32, 4, "box")); b := type_info(Box(u8, 9, "other"));
    source := a.polymorph_source_struct;
    if source != b.polymorph_source_struct || source.parameters.count != 3 return 1;
    if a.parameters.count != 3 || a.specified_parameters.count != 3 return 2;
    n := a.parameters[1]; original := source.parameters[1];
    if n.type != type_info(int) || n.offset_into_constant_storage < 0 return 3;
    actual := (cast(*int) (a.constant_storage.data + n.offset_into_constant_storage)).*;
    declared := (cast(*int) (source.constant_storage.data + original.offset_into_constant_storage)).*;
    if actual != 4 || declared != 1 return 4;
    t := source.parameters[0];
    default_type := (cast(**Type_Info) (source.constant_storage.data + t.offset_into_constant_storage)).*;
    if default_type != type_info(u8) return 5;
    label := source.parameters[2];
    text := (cast(*string) (source.constant_storage.data + label.offset_into_constant_storage)).*;
    if text != "box" return 6;
    hidden := type_info(Record);
    if hidden.alignment != align_of(Record) return 7;
    if hidden.members[0].type != type_info(*void) || (hidden.members[0].flags & 8)==0 return 8;
    if hidden.members[1].type != type_info(u8) || (hidden.members[1].flags & 8)!=0 return 9;
    return 0;
}
"#;

#[test]
fn generic_default_storage_and_relocations_native_and_wasm() {
    let files = [("/lib/prelude/module.jai", bootstrap::BOOTSTRAP)];
    backend::execute(
        "reflection-backend",
        "reflection_check",
        SOURCE,
        &files,
        None,
    );
}
