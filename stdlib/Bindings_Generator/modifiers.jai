// Canonical-model modifiers.

// Sets `PRIVATE` or `PROTECTED` on `decl` from the C++ access specifier of `cur`; public access adds nothing.
add_access_flags :: (decl: *Declaration, cur: CXCursor) {
    modifiers_add_access(decl, cur);
}

// Returns whether any argument of `func` is a `va_list`; see [is_va_list].
function_has_a_valist_arg :: (func: *Function) -> bool {
    return modifiers_has_valist(func);
}

// Marks `t` as shared between users and returns it; [add_type_flags] then copies it before changing flags.
shared_type :: (t: *CType) -> *CType {
    t.flags |= .SHARED;
    return t;
}

// Returns `t` with `flags` added.
//
// A type marked `SHARED` is copied first, and the copy is no longer shared, so shared types are never
// modified. Other types are changed in place.
add_type_flags :: (t: *CType, flags: CType.Flags) -> *CType {
    return modifiers_type_flags(t, flags);
}

// Appends the calling-convention directives of `type` to `builder`.
//
// Writes nothing for Objective-C methods, `#cpp_method` for C++ methods, and `#c_call` otherwise unless
// `assume_foreign` is set. Adds `#cpp_return_type_is_non_pod` when the return type is a struct that
// the target ABI returns through memory.
append_calling_convention_directives :: (builder: *String_Builder, type: *Function_Type, assume_foreign := false) {
    modifiers_calling_convention(builder, type, assume_foreign);
}

// Appends `#type_info_none` or `#type_info_procedures_are_void_pointers` to `builder` when `decl` has the
// matching flag.
append_type_info_directives :: (builder: *String_Builder, decl: *Declaration) {
    if decl.decl_flags & .TYPE_INFO_NONE append(builder, " #type_info_none");
    else if decl.decl_flags & .TYPE_INFO_PROCEDURES_ARE_VOID_POINTERS append(builder, " #type_info_procedures_are_void_pointers");
}

// Sets `OMIT_FROM_OUTPUT` on `func` when one of the active [Strip_Flags] applies to it.
//
// The checks are `va_list` arguments, a missing foreign library and inlined functions. Virtual and
// Objective-C
// methods are exempt from the foreign library check. The stripped count increases, and the reason is logged
// when `log_stripped_declarations` is set.
check_if_foreign_function_should_be_omitted :: (func: *Function) {
    modifiers_check_omission(func);
}

// Returns whether `decl` is printed at global scope instead of inside its struct: extern declarations,
// operators and, when `move_struct_functions_to_global_scope` is set, functions.
should_be_moved_to_global_scope :: (decl: *Declaration) -> bool {
    if is_extern_declaration(decl) return true;
    return decl.kind == .FUNCTION && (modifiers_options().move_struct_functions_to_global_scope || is_operator(decl));
}

#scope_file
#import "Basic";

#load "../internal/developer-tools/generator-modifiers.jai";
