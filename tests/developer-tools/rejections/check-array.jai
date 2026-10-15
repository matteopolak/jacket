#import "Compiler";
C :: #import "Check";

#run {
    left: Code_Node;
    left.type = type_info(*u8);
    right: Code_Node;
    right.type = type_info([4] u8);
    assignment: Code_Binary_Operator;
    assignment.kind = .BINARY_OPERATOR;
    assignment.operator_type = cast(Operator_Type) #char "=";
    assignment.left = *left;
    assignment.right = *right;
    nodes: [1] *Code_Node = .[*assignment];
    tc: Typechecked(Code_Procedure_Body);
    tc.subexpressions = nodes;
    C.check_c_porting(tc, .{});
}

main :: () {}
