#import "Compiler";
#import "Basic";
#run { root, exprs := compiler_get_nodes(#code { a := 1 + b; }); s := code_to_string(root); print("%\n", s); }
main :: () {}
