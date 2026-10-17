#import "Basic";
C :: #import "Compiler";
V :: #import "Code_Visit";

main :: () {
    left := C.make_integer_literal(13);
    right := C.make_integer_literal(29);
    expression: C.Code_Binary_Operator;
    expression.kind = .BINARY_OPERATOR;
    expression.left = left;
    expression.right = right;
    breadth: [..] *C.Code_Node;
    depth: [..] *C.Code_Node;

    V.get_recursive_nodes_breadth_first(*expression, *breadth);
    V.get_recursive_nodes_depth_first(*expression, *depth);
    assert(breadth.count == 3 && depth.count == 3);
    assert(breadth[0] == cast(*C.Code_Node) *expression);
    assert(breadth[1] == cast(*C.Code_Node) left && breadth[2] == cast(*C.Code_Node) right);
    for depth assert(it == breadth[it_index]);

    values: [..] V.Dumb_Value;

    V.get_values(*expression, *values);
    assert(values.count == 6);
    assert(values[0].node == cast(*C.Code_Node) *expression && values[0].preorder);
    assert(values[1].node == cast(*C.Code_Node) left && values[1].preorder);
    assert(values[2].node == cast(*C.Code_Node) left && !values[2].preorder);
    assert(values[5].node == cast(*C.Code_Node) *expression && !values[5].preorder);
    header: C.Code_Procedure_Header;
    header.kind = .PROCEDURE_HEADER;
    body: C.Code_Procedure_Body;
    body.kind = .PROCEDURE_BODY;
    header.body_or_null = *body;
    children: [..] *C.Code_Node;

    V.get_subexpressions(*header, *children);
    assert(children.count == 0, "headers do not descend into their bodies");
    print("ok\n");
}
