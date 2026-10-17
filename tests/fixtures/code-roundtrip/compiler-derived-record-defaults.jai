#import "Basic";
C :: #import "Compiler";

has_default :: ($T: Type, $expected: C.Code_Node.Kind) -> bool {
    local: T;
    allocated := New(T);
    result := local.kind == expected && allocated.kind == expected;
    result = result && *local.kind == *local.base.kind;
    local.base = .{};
    result = result && local.kind == .UNINITIALIZED;
    free(allocated);

    return result;
}

verify :: () -> bool {
    return has_default(C.Code_Ident, .IDENT)
        && has_default(C.Code_If, .IF)
        && has_default(C.Code_Procedure_Call, .PROCEDURE_CALL)
        && has_default(C.Code_Binary_Operator, .BINARY_OPERATOR)
        && has_default(C.Code_For, .FOR)
        && has_default(C.Code_Procedure_Body, .PROCEDURE_BODY)
        && has_default(C.Code_Context, .CONTEXT)
        && has_default(C.Code_Literal, .LITERAL)
        && has_default(C.Code_Placeholder, .PLACEHOLDER)
        && has_default(C.Code_Directive_Context_Type, .DIRECTIVE_CONTEXT_TYPE)
        && has_default(C.Code_Directive_Through, .DIRECTIVE_THROUGH);
}

#assert verify();

BINARY_CODE :: #run -> Code {
    source := #code 6 + 7;
    old := cast(*C.Code_Binary_Operator) C.compiler_get_nodes(source);
    node := New(C.Code_Binary_Operator);
    node.operator_type = old.operator_type;
    node.left = old.left;
    node.right = old.right;

    return C.compiler_get_code(node, source);
};

LOOP_CODE :: #run -> Code {
    source := #code for index: 0..2 {
        return index + 5;
    };

    old := cast(*C.Code_For) C.compiler_get_nodes(source);
    node := New(C.Code_For);
    node.iteration_expression = old.iteration_expression;
    node.iteration_expression_right = old.iteration_expression_right;
    node.ident_it = old.ident_it;
    node.ident_it_index = old.ident_it_index;
    node.block = old.block;

    return C.compiler_get_code(node, source);
};

sum :: () -> s64 {
    return #insert BINARY_CODE;
}

loop :: () -> s64 {
    #insert LOOP_CODE;
    return -1;
}

#assert sum() == 13 && loop() == 5;

main :: () -> int {
    if !verify() || sum() != 13 || loop() != 5 return 1;
    return 0;
}
