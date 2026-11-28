// Public data contract from the specification; private algorithms are independent.

// The kind of node an [Expression] is; each value but `NONE` names the struct a pointer to it can be cast to.
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

// The width and category of a primitive [CType]; see [CType.number_flags].
//
// `SIGNED`, `FLOAT` and `CHAR` combine with one of the width flags.
Number_Flags :: enum_flags u8 {
    _8BIT;
    _16BIT;
    _32BIT;
    _64BIT;
    SIGNED;
    FLOAT;
    CHAR;
}

// Properties of a [Declaration] that affect how it is checked and printed; see [Declaration.decl_flags].
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

// Selects which kinds of declarations the generator drops from its output; see
// [Generate_Bindings_Options.strip_flags].
Strip_Flags :: enum_flags {
    CONSTRUCTORS;
    DESTRUCTORS;
    FUNCTIONS_WITH_VALIST;
    SYMBOLS_WITH_UNKNOWN_FOREIGN_LIBS;
    INLINED_FUNCTIONS;
}

// Selects the scopes in which the generator mimics the source's spacing between declarations; `ALL` selects
// every scope.
Mimic_Spacing_Flags :: enum_flags {
    GLOBAL;
    STRUCT;
    ENUM;
    NAMESPACE;
    VTABLE;
    ALL :: GLOBAL | STRUCT | ENUM | NAMESPACE | VTABLE;
}

// Options for printing a single declaration.
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

// Which form of a function argument is being printed: the outer wrapper, the foreign declaration, or the call
// inside the wrapper.
Arg_Print_Mode :: enum {
    OUTER_WRAPPER;
    FOREIGN_FUNC_DECLARATION;
    INNER_WRAPPER_CALL;
}

// What a [Declaration_Visitor] returns: `RECURSE` goes on into the declaration's children and `STOP` skips
// them. Either way the walk continues with the next sibling.
Declaration_Visit_Result :: enum {
    STOP;
    RECURSE;
}
