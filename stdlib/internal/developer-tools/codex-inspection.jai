get_file_and_module :: (node: *Code_Node) -> (file: *Message_File, import: *Message_Import) {
    if !node || !node.enclosing_load return null, null;
    return node.enclosing_load, node.enclosing_load.enclosing_import;
}

is_global_ident :: (ident: *Code_Ident) -> bool {
    return ident != null && ident.resolved_declaration != null && (ident.resolved_declaration.flags & .IS_GLOBAL) != 0;
}

#scope_file
#import "Compiler";
