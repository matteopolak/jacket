C :: #import "Compiler";

#run {
    code :: #code value;
    root := C.compiler_get_nodes(code);
    root.kind = .LITERAL;
    C.compiler_get_code(root, code);
}

main :: () {}
