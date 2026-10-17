// Canonical CodeView type records, including common using base storage.
Codeview_Builtin :: struct {
    using base: Codeview_Type = .{ kind = .BUILTIN };
    id: Type_Id;
}

Codeview_Struct :: struct {
    using base: Codeview_Type = .{ kind = .STRUCTURE };
    num_fields: u16;
    properties: Struct_Properties;
    field_list: Type_Id;
    derived: Type_Id;
    vshape: u32;
    size: u64;
    name: string;
    unique_name: string;
}

Codeview_Enum :: struct {
    using base: Codeview_Type = .{ kind = .ENUM };
    num_fields: u16;
    properties: Struct_Properties;
    type: Type_Id;
    field_list: Type_Id;
    name: string;
    unique_name: string;
}

Codeview_Array :: struct {
    using base: Codeview_Type = .{ kind = .ARRAY };
    element_type: Type_Id;
    index_type: Type_Id;
    size: u64;
    name: string;
}

Codeview_Pointer :: struct {
    using base: Codeview_Type = .{ kind = .POINTER };
    type: Type_Id;
    flags: u32;
}

Codeview_Procedure :: struct {
    using base: Codeview_Type = .{ kind = .PROCEDURE };
    return_type: Type_Id;
    calling_convention: u8;
    attributes: u8;
    num_arguments: u16;
    arguments: Type_Id;
}

Codeview_Method_Function :: struct {
    using base: Codeview_Type = .{ kind = .MFUNCTION };
    return_type: Type_Id;
    class_type: Type_Id;
    this_type: Type_Id;
    calling_convention: u8;
    attributes: u8;
    num_arguments: u16;
    arguments: Type_Id;
    this_adjuster: s32;
}

Codeview_Overloaded_Method :: struct {
    using base: Codeview_Type = .{ kind = .METHOD };
    count: u16;
    list_type: Type_Id;
    name: string;
}

Codeview_Method_List :: struct {
    using base: Codeview_Type = .{ kind = .METHODLIST };
    elements: [..] Codeview_Method;
}

Codeview_Method :: struct {
    using base: Codeview_Type = .{ kind = .ONEMETHOD };
    attributes: u16;
    procedure_type: Type_Id;
    base_offset: u32;
    name: string;
}

Codeview_List :: struct {
    using base: Codeview_Type = .{ kind = .FIELDLIST };
    previous: Type_Id;
    members: [..] Codeview_Member;
    overloaded_methods: [..] Codeview_Overloaded_Method;
    methods: [..] Codeview_Method;
    enumerates: [..] Codeview_Enumerate;
    nested: [..] Codeview_Nested_Type;
}

Codeview_Arguments :: struct {
    using base: Codeview_Type = .{ kind = .ARGLIST };
    arguments: [] Type_Id;
}

Codeview_String_Id :: struct {
    using base: Codeview_Type = .{ kind = .STRING_ID };
    id: u32;
    name: string;
}

Codeview_Udt_Source_Line :: struct {
    using base: Codeview_Type = .{ kind = .UDT_SRC_LINE };
    type: Type_Id;
    src: Type_Id;
    line: u32;
}

Codeview_Function_Id :: struct {
    using base: Codeview_Type = .{ kind = .FUNC_ID };
    scope: Type_Id;
    type: Type_Id;
    name: string;
    undocumented: u64;
}

Codeview_Alias :: struct {
    using base: Codeview_Type = .{ kind = .ALIAS };
    type: Type_Id;
    name: string;
}

Codeview_Typeserver2 :: struct {
    using base: Codeview_Type = .{ kind = .TYPESERVER2 };
    sig70: Guid;
    age: u32;
    name: string;
}

Codeview_Modifier :: struct {
    using base: Codeview_Type = .{ kind = .MODIFIER };
    type: Type_Id;
    modifier: Modifier_Type;
    Modifier_Type :: enum_flags u16 {
        CONST :: 0x01;
        VOLATILE :: 0x02;
        UNALINGED :: 0x04;
    }
}

Codeview_Member :: struct {
    using base: Codeview_Type = .{ kind = .MEMBER };
    access: u16;
    type: Type_Id;
    offset: u64;
    name: string;
}

Codeview_Enumerate :: struct {
    using base: Codeview_Type = .{ kind = .ENUMERATE };
    access: u16;
    value: u64;
    name: string;
}

Codeview_Nested_Type :: struct {
    using base: Codeview_Type = .{ kind = .NESTTYPE };
    type: Type_Id;
    name: string;
}

Struct_Properties :: enum_flags u16 {
    PACKED :: 0x0001;
    CTOR :: 0x0002;
    OVLOPS :: 0x0004;
    IS_NESTED :: 0x0008;
    NESTED_TYPES :: 0x0010;
    OP_ASSIGN :: 0x0020;
    OP_CAST :: 0x0040;
    FWDREF :: 0x0080;
    SCOPED :: 0x0100;
    UNIQUE_NAME :: 0x0200;
    SEALED :: 0x0400;
    HFA_OTHER :: 0x1800;
    HFA_FLOAT :: 0x0800;
    HFA_DOUBLE :: 0x1000;
    INTRINSIC :: 0x2000;
    MOCOM_UDT_INTERFACE :: 0xC000;
    MOCOM_UDT_REF :: 0x4000;
    MOCOM_UDT_VALUE :: 0x8000;
}

Function_Attributes :: enum_flags u8 {
    CXX_RETURN_UDT :: 0x01;
    CTOR :: 0x02;
    CTOR_V_BASE :: 0x03;
}

Codeview_Access :: enum u16 {
    NONE :: 0;
    PUBLIC :: 3;
}

Codeview_Builtin_Type :: enum {
    NOTYPE :: 0x0000;
    ABS :: 0x0002;
    SEGMENT :: 0x0002;
    VOID :: 0x0003;
    HRESULT :: 0x0008;
    CHAR :: 0x0010;
    SHORT :: 0x0011;
    LONG :: 0x0012;
    QUAD :: 0x0013;
    UCHAR :: 0x0020;
    USHORT :: 0x0021;
    ULONG :: 0x0022;
    UQUAD :: 0x0023;
    FLOAT32 :: 0x0040;
    FLOAT64 :: 0x0041;
    FLOAT16 :: 0x0046;
    S8 :: 0x0068;
    U8 :: 0x0069;
    RCHAR :: 0x0070;
    WCHAR :: 0x0071;
    CHAR16 :: 0x007a;
    CHAR32 :: 0x007b;
    S16 :: 0x0072;
    U16 :: 0x0073;
    S32 :: 0x0074;
    U32 :: 0x0075;
    S64 :: 0x0076;
    U64 :: 0x0077;
    BOOL8 :: 0x0030;
    BOOL16 :: 0x0031;
    BOOL32 :: 0x0032;
    BOOL64 :: 0x0033;
}

Codeview_Pointer_Type :: enum u8 {
    NEAR :: 0x00;
    FAR :: 0x01;
    HUGE :: 0x02;
    BASE_SEG :: 0x03;
    BASE_VAL :: 0x04;
    BASE_SEGVAL :: 0x05;
    BASE_ADDR :: 0x06;
    BASE_SEGADDR :: 0x07;
    BASE_TYPE :: 0x08;
    BASE_SELF :: 0x09;
    NEAR32 :: 0x0a;
    FAR32 :: 0x0b;
    _64 :: 0x0c;
    UNUSEDPTR :: 0x0d;
}

Guid :: struct {
    data1: u32;
    data2: u16;
    data3: u16;
    data4: [8] u8;
}
