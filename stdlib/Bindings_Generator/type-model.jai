// Public data contract from the specification; private algorithms are independent.

CType :: struct {
    hardcoded_jai_string: string;
    pointer_to: *CType;
    type_of_enum: *Enum;
    type_of_typedef: *Typedef;
    type_of_struct: *Struct;
    type_of_declaration: *Declaration;
    array_element_type: *CType;
    array_element_count := -1;
    array_element_count_expression: *Expression;
    type_of_function: *Function_Type;
    template_instantiation_params: [..] *Expression;
    size: s64 = xx CXTypeLayoutError.Invalid;
    alignment: s64 = -1;
    number_flags: Number_Flags;
    flags: Flags;
    Flags :: enum_flags u8 {
        REFERENCE;
        CONST;
        VOLATILE;
        INSERTED;
        SHARED;
        VOID;
        BOOL
    }
    comment: string;
}

Function_Type :: struct {
    arguments: [..] *Declaration;
    return_type: *CType;
    flags: Flags;
    Flags :: enum_flags {
        ADD_C_TYPE_DETAILS;
        INLINE;
        OBJC_METHOD;
        OBJC_CLASS_METHOD;
        VARIADIC;
        CPP_METHOD;
        CONSTRUCTOR;
        COPY_CONSTRUCTOR;
        MOVE_CONSTRUCTOR;
        DESTRUCTOR
    }
}

Expression :: struct {
    kind: C_Kind;
}

Type_Reference :: struct {
    #as using _expression: Expression;
    type: *CType;
}

Literal :: struct {
    #as using _expression: Expression;
    kind = .LITERAL;
    literal_kind: enum int {
        NULL :: 0;
        INTEGER :: 1;
        UNSIGNED_INTEGER :: 2;
        FLOAT :: 3;
        STRING :: 4;
        RAW_CPP_TOKENS :: 5;
        ENUM :: 6;
        MACRO :: 7;
        IDENTIFIER :: 8;
    };
    union {
        int_value: s64;
        uint_value: u64;
        float_value: float64;
        string_value: string;
        macro_value: Macro_Value;
        struct {
            enum_type: *Enum;
            enum_value: *Declaration;
        }
        identifier: *Declaration;
    }
    raw_value: string;
    template_instantiation_params: [..] *Expression;
    location: Location;
}

Macro_Value :: struct {
    tokens: [] CXToken;
}

Unary_Operator :: struct {
    #as using _expression: Expression;
    op: string;
    child: *Expression;
}

Binary_Operator :: struct {
    #as using _expression: Expression;
    op: string;
    lhs: *Expression;
    rhs: *Expression;
}

Paren_Expression :: struct {
    #as using _expression: Expression;
    child: *Expression;
}

Struct_Literal :: struct {
    #as using _expression: Expression;
    type_name: string;
    values: [] *Expression;
}
