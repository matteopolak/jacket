// Public data contract from the specification; private algorithms are independent.

C_Kind :: enum {
    NONE;
    DECLARATION;
    FUNCTION;
    ENUM;
    TYPEDEF;
    STRUCT;
    BITFIELD;
    NAMESPACE;
    MACRO_DEFINITION;
    TYPE_REFERENCE;
    LITERAL;
    UNARY_OPERATOR;
    BINARY_OPERATOR;
    STRUCT_LITERAL;
    PAREN_EXPRESSION;
}

Number_Flags :: enum_flags u8 {
    _8BIT;
    _16BIT;
    _32BIT;
    _64BIT;
    SIGNED;
    FLOAT;
    CHAR;
}

Declaration_Flags :: enum_flags {
    NEEDS_ARGUMENT_WRAPPER;
    NEEDS_PRINTF_WRAPPER;
    POINTER_TO_VALUE;
    OMIT_FROM_OUTPUT;
    OMIT_FROM_CHECKS;
    NEEDS_DEFAULT_ARGUMENT_CAST;
    IS_CONST;
    IS_METADATA;
    QUALIFIER_AS;
    QUALIFIER_USING;
    TYPE_INFO_NONE;
    TYPE_INFO_PROCEDURES_ARE_VOID_POINTERS;
    NO_SPACING;
    TEMPLATE;
    DEPRECATED;
    PROTECTED;
    PRIVATE;
}

Strip_Flags :: enum_flags {
    CONSTRUCTORS;
    DESTRUCTORS;
    FUNCTIONS_WITH_VALIST;
    SYMBOLS_WITH_UNKNOWN_FOREIGN_LIBS;
    INLINED_FUNCTIONS;
}

Mimic_Spacing_Flags :: enum_flags {
    GLOBAL;
    STRUCT;
    ENUM;
    NAMESPACE;
    VTABLE;
    ALL :: GLOBAL | STRUCT | ENUM | NAMESPACE | VTABLE;
}

Print_Decl_Flags :: enum_flags {
    NO_SEMICOLON;
    NO_INDENT;
    IN_FUNCTION_ARGS;
    IN_FUNCTION_RETURN;
    IS_GLOBAL;
    NO_ARGUMENT_DEFAULTS;
    KEEP_POINTER_TO_VALUE;
    C_TYPE_DETAILS;
    NO_PATH;
}

Arg_Print_Mode :: enum {
    OUTER_WRAPPER;
    FOREIGN_FUNC_DECLARATION;
    INNER_WRAPPER_CALL;
}

Declaration_Visit_Result :: enum {
    STOP;
    RECURSE;
}
