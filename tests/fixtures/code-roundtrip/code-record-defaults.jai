#import "Basic";
C :: #import "Compiler";

verify :: () -> bool {
    base: C.Code_Node;
    local: C.Code_Ident;
    allocated := New(C.Code_Ident);
    call: C.Code_Procedure_Call;
    branch: C.Code_If;
    result := base.kind == .UNINITIALIZED && local.kind == .IDENT && allocated.kind == .IDENT;
    result = result && call.kind == .PROCEDURE_CALL && branch.kind == .IF;
    local.name = "answer";
    allocated.name = "other";
    result = result && local.kind == .IDENT && allocated.kind == .IDENT;
    result = result && *local.kind == *local.base.kind;
    result = result && type_of(local.kind) == C.Code_Node.Kind;
    local.base = .{};
    result = result && local.kind == .UNINITIALIZED;
    free(allocated);

    return result;
}

#assert verify();

SYNTAX_OK :: #run -> bool {
    local: C.Code_Ident;
    local.name = "answer";
    code := C.compiler_get_code(*local);
    parsed := C.compiler_get_nodes(code);

    return parsed.kind == .IDENT && (cast(*C.Code_Ident) parsed).name == "answer";
};

#assert SYNTAX_OK;

main :: () -> int {
    if !verify() return 1;
    return 0;
}
