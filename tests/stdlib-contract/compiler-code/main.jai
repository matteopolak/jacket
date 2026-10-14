#import "Basic";
C :: #import "Compiler";

#run {
    code := #code (13 + 29);
    root, expressions := C.compiler_get_nodes(code);

    assert(root != null && root.kind == .BINARY_OPERATOR);
    binary := cast(*C.Code_Binary_Operator) root;

    assert((cast(*C.Code_Literal) binary.left)._s64 == 13);
    assert((cast(*C.Code_Literal) binary.right)._s64 == 29);
    assert(expressions.count >= 3);
    copied := C.compiler_get_code(root, code);
    copied_root := C.compiler_get_nodes(copied);

    assert(copied_root.kind == .BINARY_OPERATOR);
    assert(C.get_type(type_info(s32)) == s32);
    status, type := C.get_root_type(#code,null);

    assert(status == .INPUT_IS_CODE_NULL);
}

main :: () {
    print("ok\n");
}
