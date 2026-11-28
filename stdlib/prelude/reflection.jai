// Which kind of type a [Type_Info] describes, and so which `Type_Info_` struct it starts.
// `BOOL`, `VOID`, `ANY`, `TYPE`, `CODE` and the untyped tags have no struct beyond [Type_Info].
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

// What the program knows about a type at run time, as `type_info(T)` returns it. Cast it to the
// struct its `type` tag names to see more.
//
// ```jai
// info := type_info(u16);
// assert(info.type == .INTEGER);
// assert(info.runtime_size == 2);
// assert(!(cast(*Type_Info_Integer) info).signed);
// ```
Type_Info :: struct {
    type: Type_Info_Tag;
    runtime_size: s64; // Bytes a value takes; -1 while the type is incomplete or polymorphic.
}

// An integer type such as `u8` or `s64`; `signed` says which kind.
Type_Info_Integer :: struct {
    using #as info: Type_Info;
    signed: bool;
}

// `float32` or `float64`; tell them apart by `runtime_size`.
Type_Info_Float :: struct {
    using #as info: Type_Info;
}

// The `string` type.
Type_Info_String :: struct {
    using #as info: Type_Info;
}

// A pointer type: `pointer_to` is what it points at.
Type_Info_Pointer :: struct {
    using #as info: Type_Info;

    pointer_to: *Type_Info;
}

// An array type: fixed `[N] T`, view `[] T` or resizable `[..] T`.
Type_Info_Array :: struct {
    using #as info: Type_Info;

    element_type: *Type_Info;

    array_type: Array_Type;

    array_count: s64; // `N` for a fixed array; -1 for views and resizable arrays.

    Array_Type :: enum u16 {
        FIXED :: 0;
        VIEW :: 1;
        RESIZABLE :: 2;
    }
}

// A procedure type: its argument and return types, and how it is called.
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

// The compiler's progress on an enum: `.INCOMPLETE` while its members are still being resolved.
Enum_Status_Flags :: enum_flags u16 {
    INCOMPLETE :: 0x1;
}

// How an enum was declared.
Enum_Type_Flags :: enum_flags u16 {
    FLAGS :: 0x1; // Declared with `enum_flags`.
    COMPLETE :: 0x2; // Marked `#complete`.
    SPECIFIED :: 0x4; // Marked `#specified`.
}

// An enum type. `names[i]` is the member with value `values[i]`, in declaration order.
//
// ```jai
// Color :: enum {
//     RED;
//     GREEN;
//     BLUE :: 7;
// }
// info := type_info(Color);
// assert(info.name == "Color");
// assert(info.names[2] == "BLUE" && info.values[2] == 7);
// ```
Type_Info_Enum :: struct {
    using #as info: Type_Info;

    name: string;
    internal_type: *Type_Info_Integer; // The integer type values are stored in.

    names: [] string;
    values: [] s64;

    status_flags: Enum_Status_Flags;
    enum_type_flags: Enum_Type_Flags;
}

// The compiler's progress on a struct: `.INCOMPLETE` while its members are still being resolved;
// `.LOCAL` for a struct declared inside a procedure.
Struct_Status_Flags :: enum_flags u32 {
    INCOMPLETE :: 0x1;
    LOCAL :: 0x4;
}

// What a struct's declaration asked for, one flag per keyword or directive.
Struct_Textual_Flags :: enum_flags u32 {
    FOREIGN :: 0x1;
    UNION :: 0x2; // Declared with `union`.
    NO_PADDING :: 0x4; // `#no_padding`.
    TYPE_INFO_NONE :: 0x8; // `#type_info_none`: members are left out of the type info.
    TYPE_INFO_NO_SIZE_COMPLAINT :: 0x10; // `#type_info_no_size_complaint`.
    TYPE_INFO_PROCEDURES_ARE_VOID_POINTERS :: 0x20; // `#type_info_procedures_are_void_pointers`.
    UNION_IS_TAGGED :: 0x40;
}

// Facts the compiler worked out about a struct.
Struct_Nontextual_Flags :: enum_flags u32 {
    NOT_INSTANTIABLE :: 0x4;
    ALL_MEMBERS_UNINITIALIZED :: 0x40; // Every member is declared `---`.
    POLYMORPHIC :: 0x100; // Has parameters that are not filled in yet.
}

// One member of a struct, a constant or nested declaration among them, as listed in
// [Type_Info_Struct.members].
Type_Info_Struct_Member :: struct {
    name: string;
    type: *Type_Info;
    offset_in_bytes: s64; // Negative for constants, which take no space in a value.

    flags: Flags;

    notes: [] string; // The `@` notes after the member, without the `@`.

    offset_into_constant_storage: s64 = -1; // Where a constant's value is in [Type_Info_Struct.constant_storage]; -1 when not stored.

    Flags :: enum_flags u32 {
        CONSTANT :: 0x1; // Declared with `::`.
        IMPORTED :: 0x2;
        USING :: 0x4; // Declared with `using`.
        PROCEDURE_WITH_VOID_POINTER_TYPE_INFO :: 0x8;
        AS :: 0x10;
        OVERLAY :: 0x20;
    }
}

// Which union member a tag value selects, in a union marked as tagged.
Type_Info_Tagged_Union_Binding :: struct {
    constant_value: u64;
    member_index: s32;
}

// A struct or union type: its members, flags and layout.
//
// `members` lists fields, constants and nested declarations in declaration order. Fields have
// non-negative offsets.
//
// ```jai
// Point :: struct {
//     x: s32;
//     y: s32;
//     ORIGIN :: 0;
// }
// info := type_info(Point);
// assert(info.name == "Point");
// assert(info.members[1].name == "y" && info.members[1].offset_in_bytes == 4);
// assert(info.members[2].offset_in_bytes < 0);
// ```
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

    polymorph_source_struct: *Type_Info_Struct; // The polymorphic struct this one was made from, or null.

    initializer: (*void) #no_context; // Sets a value to the struct's defaults.

    constant_storage: [] u8;
    notes: [] string;
}

// How a [Type_Info_Variant] was declared.
Type_Info_Variant_Flags :: enum_flags u32 {
    DISTINCT :: 0x1; // `#type,distinct`.
    ISA :: 0x2; // `#type,isa`.
}

// A named type made from another with `#type,distinct` or `#type,isa`.
Type_Info_Variant :: struct {
    using #as info: Type_Info;

    name: string;
    variant_of: *Type_Info;
    variant_flags: Type_Info_Variant_Flags;
}
