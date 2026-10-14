// Deliberately scoped test bootstrap: these are the reviewed fields needed by
// these tests, not a substitute for integrated prelude verification.
pub const BOOTSTRAP: &str = r#"
Type_Info :: struct { type: u32; runtime_size: s64; }
Type_Info_Integer :: struct { using #as info: Type_Info; signed: bool; }
Type_Info_String :: struct { using #as info: Type_Info; }
Type_Info_Float :: struct { using #as info: Type_Info; }
Type_Info_Void :: struct { using #as info: Type_Info; }
Type_Info_Pointer :: struct { using #as info: Type_Info; pointer_to: *Type_Info; }
Type_Info_Struct_Member :: struct {
    name: string; type: *Type_Info; offset_in_bytes: s64;
    flags: u32; notes: [] string; offset_into_constant_storage: s64 = -1;
}
Type_Info_Struct :: struct {
    using #as info: Type_Info; name: string;
    parameters: [] Type_Info_Struct_Member; specified_parameters: [] Type_Info_Struct_Member;
    members: [] Type_Info_Struct_Member;
    status_flags: u16; nontextual_flags: u16; textual_flags: u16; alignment: s32;
    polymorph_source_struct: *Type_Info_Struct;
    initializer: (*void) #no_context; constant_storage: [] u8; notes: [] string;
}
"#;
