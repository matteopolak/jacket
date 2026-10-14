#import "Basic";
C :: #import "Compiler";
P :: #import "Program_Print";

main :: () {
    left := C.make_integer_literal(-13);
    right := C.make_integer_literal(29);
    binary: C.Code_Binary_Operator;
    binary.kind = .BINARY_OPERATOR;
    binary.operator_type = cast(C.Operator_Type) #char "+";
    binary.left = left;
    binary.right = right;
    builder: String_Builder;
    init_string_builder(*builder);
    assert(P.print_expression(*builder, *binary));
    assert(builder_to_string(*builder) == "(-13 + 29)");
    declaration: C.Code_Declaration;
    declaration.kind = .DECLARATION;
    declaration.name = "value";
    declaration.flags = .IS_CONSTANT;
    declaration.expression = left;
    assert(P.print_declaration(*declaration) == "value:: -13");
    unsupported: C.Code_Asm;
    unsupported.kind = .ASM;
    assert(!P.print_expression(*builder, *unsupported));
    print("ok\n");
}
