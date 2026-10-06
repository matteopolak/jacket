#import "Compiler";
#import "Basic";
#run { s := code_to_string(null); print("%\n", s); n: Code_Literal; n.kind = .LITERAL; n.value_type = .STRING; print_expression(null, *n); }
main :: () {}
