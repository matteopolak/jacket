C :: #import "Compiler";

#run {
    code :: #code f(a + b);
    call := cast(*C.Code_Procedure_Call) C.compiler_get_nodes(code);
    sum := cast(*C.Code_Binary_Operator) call.arguments_unsorted[0].expression;
    sum.operator_type = #char "=";
    C.compiler_get_code(call, code);
}

main :: () {}
