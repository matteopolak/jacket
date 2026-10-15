#import "Basic";
#import "Compiler";
C :: #import "Codex";
M :: #import "Math";

global_value: s32;

#run {
    root := compiler_get_nodes(#code global_value);

    assert(root.kind == .IDENT);
    assert(C.is_global_ident(cast(*Code_Ident) root));
    file, module := C.get_file_and_module(root);

    assert(file != null && module == file.enclosing_import);
}

main :: () {
    assert(type_info(C.Math.Vector3) == type_info(M.Vector3));
    declaration: Code_Declaration;
    ident: Code_Ident;
    ident.resolved_declaration = *declaration;
    assert(!C.is_global_ident(*ident));
    declaration.flags = .IS_GLOBAL;
    assert(C.is_global_ident(*ident));
    declaration.flags |= .IS_CONSTANT;
    assert(C.is_global_ident(*ident));
    null_file, null_module := C.get_file_and_module(null);

    assert(null_file == null && null_module == null);
    print("ok\n");
}
