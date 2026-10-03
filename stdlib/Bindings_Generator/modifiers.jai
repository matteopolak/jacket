// Canonical-model modifiers; policies: docs/generator-modifiers.md.
#load "active-context.jai";

add_access_flags :: (decl: *Declaration, cur: CXCursor) {
    modifiers_add_access(decl, cur);
}

function_has_a_valist_arg :: (func: *Function) -> bool {
    return modifiers_has_valist(func);
}

shared_type :: (t: *CType) -> *CType {
    t.flags |= .SHARED;
    return t;
}

add_type_flags :: (t: *CType, flags: CType.Flags) -> *CType {
    return modifiers_type_flags(t, flags);
}

append_calling_convention_directives :: (builder: *String_Builder, type: *Function_Type, assume_foreign := false) {
    modifiers_calling_convention(builder, type, assume_foreign);
}

append_type_info_directives :: (builder: *String_Builder, decl: *Declaration) {
    if decl.decl_flags & .TYPE_INFO_NONE append(builder, " #type_info_none");
    else if decl.decl_flags & .TYPE_INFO_PROCEDURES_ARE_VOID_POINTERS append(builder, " #type_info_procedures_are_void_pointers");
}

check_if_foreign_function_should_be_omitted :: (func: *Function) {
    modifiers_check_omission(func);
}

should_be_moved_to_global_scope :: (decl: *Declaration) -> bool {
    if is_extern_declaration(decl) return true;
    return decl.kind == .FUNCTION && (modifiers_options().move_struct_functions_to_global_scope || is_operator(decl));
}

#scope_file
#import "Basic";

#load "../internal/developer-tools/generator-modifiers.jai";
