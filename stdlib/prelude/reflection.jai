// Canonical bootstrap reflection declarations.
Type_Info_Tag :: enum u32 {
    INTEGER :: 0;
    FLOAT :: 1;
    BOOL :: 2;
    STRING :: 3;
    POINTER :: 4;
    PROCEDURE :: 5;
    VOID :: 6;
    STRUCT :: 7;
    ARRAY :: 8;
    OVERLOAD_SET :: 9;
    ANY :: 10;
    ENUM :: 11;
    POLYMORPHIC_VARIABLE :: 12;
    TYPE :: 13;
    CODE :: 14;
    UNTYPED_LITERAL :: 15;
    UNTYPED_ENUM :: 16;

    VARIANT :: 18;
}

Type_Info :: struct {
    type: Type_Info_Tag;
    runtime_size: s64;
}

Type_Info_Integer :: struct {
    using #as info: Type_Info;
    signed: bool;
}

Type_Info_Float :: struct {
    using #as info: Type_Info;
}

Type_Info_String :: struct {
    using #as info: Type_Info;
}

Type_Info_Pointer :: struct {
    using #as info: Type_Info;

    pointer_to: *Type_Info;
}

Type_Info_Array :: struct {
    using #as info: Type_Info;

    element_type: *Type_Info;

    array_type: Array_Type;

    array_count: s64;

    Array_Type :: enum u16 {
        FIXED :: 0;
        VIEW :: 1;
        RESIZABLE :: 2;
    }
}

Type_Info_Procedure :: struct {
    using #as info: Type_Info;

    argument_types: [] *Type_Info;
    return_types: [] *Type_Info;

    procedure_flags: Flags;

    Flags :: enum_flags u32 {
        IS_ELSEWHERE :: 0x1;
        IS_COMPILE_TIME_ONLY :: 0x2;
        IS_POLYMORPHIC :: 0x4;
        HAS_NO_CONTEXT :: 0x8;
        IS_C_CALL :: 0x20;
        IS_INTRINSIC :: 0x80;
        IS_SYMMETRIC :: 0x100;

        IS_CPP_METHOD :: 0x1000_0000;
        HAS_CPP_NON_POD_RETURN_TYPE :: 0x2000_0000;
    }
}

Enum_Status_Flags :: enum_flags u16 {
    INCOMPLETE :: 0x1;
}

Enum_Type_Flags :: enum_flags u16 {
    FLAGS :: 0x1;
    COMPLETE :: 0x2;
    SPECIFIED :: 0x4;
}

Type_Info_Enum :: struct {
    using #as info: Type_Info;

    name: string;
    internal_type: *Type_Info_Integer;

    names: [] string;
    values: [] s64;

    status_flags: Enum_Status_Flags;
    enum_type_flags: Enum_Type_Flags;
}

Struct_Status_Flags :: enum_flags u32 {
    INCOMPLETE :: 0x1;
    LOCAL :: 0x4;
}

Struct_Textual_Flags :: enum_flags u32 {
    FOREIGN :: 0x1;
    UNION :: 0x2;
    NO_PADDING :: 0x4;
    TYPE_INFO_NONE :: 0x8;
    TYPE_INFO_NO_SIZE_COMPLAINT :: 0x10;
    TYPE_INFO_PROCEDURES_ARE_VOID_POINTERS :: 0x20;
    UNION_IS_TAGGED :: 0x40;
}

Struct_Nontextual_Flags :: enum_flags u32 {
    NOT_INSTANTIABLE :: 0x4;
    ALL_MEMBERS_UNINITIALIZED :: 0x40;
    POLYMORPHIC :: 0x100;
}

Type_Info_Struct_Member :: struct {
    name: string;
    type: *Type_Info;
    offset_in_bytes: s64;

    flags: Flags;

    notes: [] string;

    offset_into_constant_storage: s64 = -1;

    Flags :: enum_flags u32 {
        CONSTANT :: 0x1;
        IMPORTED :: 0x2;
        USING :: 0x4;
        PROCEDURE_WITH_VOID_POINTER_TYPE_INFO :: 0x8;
        AS :: 0x10;
        OVERLAY :: 0x20;
    }
}

Type_Info_Tagged_Union_Binding :: struct {
    constant_value: u64;
    member_index: s32;
}

Type_Info_Struct :: struct {
    using #as info: Type_Info;

    name: string;

    parameters: [] Type_Info_Struct_Member;

    specified_parameters: [] Type_Info_Struct_Member;
    members: [] Type_Info_Struct_Member;

    tagged_union_bindings: [] Type_Info_Tagged_Union_Binding;

    status_flags: Struct_Status_Flags;
    nontextual_flags: Struct_Nontextual_Flags;
    textual_flags: Struct_Textual_Flags;

    alignment: s32;

    polymorph_source_struct: *Type_Info_Struct;

    initializer: (*void) #no_context;

    constant_storage: [] u8;
    notes: [] string;
}

Type_Info_Variant_Flags :: enum_flags u32 {
    DISTINCT :: 0x1;
    ISA :: 0x2;
}

Type_Info_Variant :: struct {
    using #as info: Type_Info;

    name: string;
    variant_of: *Type_Info;
    variant_flags: Type_Info_Variant_Flags;
}
