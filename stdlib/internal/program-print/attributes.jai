// Owned serialization metadata; no additions to the public Code_* layouts.
// Private syntax attributes a record carries beyond the public Code_* layouts. The Rust
// compiler declares the same members in `crates/jaic/src/builtin_keys.rs`; keep both in step.
Syntax_Key :: enum u16 {
    ALIAS :: 0;
    ALIAS_EXPRESSION :: 1;
    ALIGNMENT :: 2;
    ASM_ITEMS :: 3;
    ASM_KIND :: 4;
    ASM_PRESENT :: 5;
    ASSERT_ARGS :: 6;
    ASSERT_MESSAGE :: 7;
    BACKTICK_LABEL :: 8;
    BIND_LABEL :: 9;
    BROADCAST :: 10;
    BUILTIN_NAME :: 11;
    CALL_HINT :: 12;
    CALLER_CODE :: 13;
    CLASS :: 14;
    COLON :: 15;
    CONTEXT_BARE :: 16;
    DEPRECATION_NAMED :: 17;
    DIRECTIVE_ARGS :: 18;
    DIRECTIVE_NAME :: 19;
    EXPORT_NAME :: 20;
    EXPORT_NAMED :: 21;
    FEATURES :: 22;
    FILTER :: 23;
    FILTER_ARGUMENTS :: 24;
    FILTER_EXPRESSION :: 25;
    FILTER_KIND :: 26;
    FLAGS :: 27;
    FOREIGN :: 28;
    FOREIGN_LIBRARY :: 29;
    FOREIGN_NAMED :: 30;
    FOREIGN_SYMBOL :: 31;
    IMPORT_KIND :: 32;
    INCOMPLETE :: 33;
    INITIAL_EXPRESSION :: 34;
    INTERFACE :: 35;
    ITERATOR :: 36;
    LABEL :: 37;
    LAMBDA_BODY :: 38;
    MASK :: 39;
    MNEMONIC :: 40;
    NAME :: 41;
    NEGATE :: 42;
    OPERAND :: 43;
    OPERANDS :: 44;
    ORIGINAL_CONTEXT :: 45;
    ORIGINAL_KIND :: 46;
    PIN :: 47;
    POLY_BAKED :: 48;
    PROGRAM_EXPORT :: 49;
    RESTRICTION :: 50;
    RETURN_MUST :: 51;
    ROUND :: 52;
    RUN_BODY :: 53;
    SCALE :: 54;
    SIZE :: 55;
    STATEMENT_NOTES :: 56;
    STATIC_FILTER :: 57;
    SUFFIX :: 58;
    TERMS :: 59;
    UNION_TAG :: 60;
    USING :: 61;
    VALUE :: 62;
    ZEROING :: 63;
    CONTEXT_ARGUMENTS :: 64;
    CONTEXT_NAMES :: 65;
}

syntax_int :: (node: *C.Code_Node, key: Syntax_Key) -> int {
    if #compile_time return __jaic_syntax_int(node, key);
    return 0;
}

syntax_string :: (node: *C.Code_Node, key: Syntax_Key) -> string {
    value: string;
    if #compile_time __jaic_syntax_string(node, key, *value);
    return value;
}

syntax_child :: (node: *C.Code_Node, key: Syntax_Key) -> *C.Code_Node {
    if #compile_time return __jaic_syntax_child(node, key, type_info(C.Code_Node));
    return null;
}

#scope_file
C :: #import "Compiler";

__jaic_syntax_int :: (node: *void, key: Syntax_Key) -> int #compiler;
__jaic_syntax_string :: (node: *void, key: Syntax_Key, result: *string) #compiler;
__jaic_syntax_child :: (node: *void, key: Syntax_Key, expected: *Type_Info) -> *C.Code_Node #compiler;
