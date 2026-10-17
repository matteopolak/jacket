C :: #import "Compiler";

#run {
    node: C.Code_Ident;
    node.name = "not_an_initialized_identifier";
    node.kind = .UNINITIALIZED;
    C.compiler_get_code(*node);
}

main :: () {}
