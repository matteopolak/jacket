// Public data contract from the specification; private algorithms are independent.

// A C type as the generator models it: a primitive, or a pointer, array, function, struct, enum or typedef
// over other types.
//
// The `pointer_to`, `array_element_type`, `type_of_function`, `type_of_struct`, `type_of_enum` and
// `type_of_typedef` fields select the form; a primitive sets `number_flags` and `size` instead. `size` and
// `alignment` are in bytes. `array_element_count` is -1 for an array without a known length.
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

// The signature of a function: its `arguments` and `return_type`; see [Function_Type.flags].
//
// `VARIADIC` marks a trailing `...`; the C++ and Objective-C flags record the kind of method.
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

// The base of every expression node. `kind` says which concrete struct a pointer to it really is.
Expression :: struct {
    kind: C_Kind;
}

// An expression that names a type.
Type_Reference :: struct {
    #as using _expression: Expression;
    type: *CType;
}

// A literal or identifier expression; `literal_kind` selects which member of the union is valid.
//
// `NULL` carries no value, `INTEGER` uses `int_value`, `UNSIGNED_INTEGER` `uint_value`, `FLOAT` `float_value`
// and `STRING` and `RAW_CPP_TOKENS` `string_value`. `ENUM` uses `enum_type` and `enum_value`, `MACRO` uses
// `macro_value`, and `IDENTIFIER` uses `identifier`. `raw_value` keeps the source spelling when one exists.
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

// The replacement tokens of an object-like macro, as libclang tokens.
Macro_Value :: struct {
    tokens: [] CXToken;
}

// A unary operator applied to `child`; `op` is its spelling, such as `-` or `sizeof`.
Unary_Operator :: struct {
    #as using _expression: Expression;
    op: string;
    child: *Expression;
}

// A binary operator `op` applied to `lhs` and `rhs`.
Binary_Operator :: struct {
    #as using _expression: Expression;
    op: string;
    lhs: *Expression;
    rhs: *Expression;
}

// A parenthesised expression.
Paren_Expression :: struct {
    #as using _expression: Expression;
    child: *Expression;
}

// A struct initializer of type `type_name` with one expression per `values` entry.
Struct_Literal :: struct {
    #as using _expression: Expression;
    type_name: string;
    values: [] *Expression;
}
