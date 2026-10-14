// Direct-child visitation preserves child order for a binary expression node.
#import "Basic";
Compiler :: #import "Compiler";
Code_Visit :: #import "Code_Visit";

main :: () {
    left := Compiler.make_integer_literal(13);
    right := Compiler.make_integer_literal(29);
    expression: Compiler.Code_Binary_Operator;
    expression.kind = .BINARY_OPERATOR;
    expression.left = left;
    expression.right = right;

    children: [..] *Compiler.Code_Node;
    Code_Visit.get_subexpressions(*expression, *children);
    assert(children.count == 2);
    assert(children[0] == cast(*Compiler.Code_Node) left);
    assert(children[1] == cast(*Compiler.Code_Node) right);
    print("ok\n");
}
