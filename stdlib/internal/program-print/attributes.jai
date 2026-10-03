// Owned serialization metadata; no additions to the public Code_* layouts.
syntax_int :: (node: *C.Code_Node, key: string) -> int {
    if #compile_time return __jaic_syntax_int(node, key);
    return 0;
}

syntax_string :: (node: *C.Code_Node, key: string) -> string {
    value: string;
    if #compile_time __jaic_syntax_string(node, key, *value);
    return value;
}

syntax_child :: (node: *C.Code_Node, key: string) -> *C.Code_Node {
    if #compile_time return __jaic_syntax_child(node, key, type_info(C.Code_Node));
    return null;
}

#scope_file
C :: #import "Compiler";

__jaic_syntax_int :: (node: *void, key: string) -> int #compiler;
__jaic_syntax_string :: (node: *void, key: string, result: *string) #compiler;
__jaic_syntax_child :: (node: *void, key: string, expected: *Type_Info) -> *C.Code_Node #compiler;
