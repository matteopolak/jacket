#scope_module
#import "Basic";

modifiers_options :: () -> *Generate_Bindings_Options {
    options := Generator_Context.active_options();
    assert(options != null, "Bindings_Generator: active options required");
    return cast(*Generate_Bindings_Options) options;
}

modifiers_generator :: () -> *Generator {
    generator := Generator_Context.active_generator();
    assert(generator != null, "Bindings_Generator: active generator required");
    return cast(*Generator) generator;
}

modifiers_add_access :: (decl: *Declaration, cur: CXCursor) {
    access := clang_getCXXAccessSpecifier(cur);
    if access == .CXXProtected decl.decl_flags |= .PROTECTED;
    if access == .CXXPrivate decl.decl_flags |= .PRIVATE;
}

modifiers_has_valist :: (func: *Function) -> bool {
    for argument: func.type.type_of_function.arguments {
        if is_va_list(argument.type) return true;
    }

    return false;
}

modifiers_type_flags :: (t: *CType, flags: CType.Flags) -> *CType {
    result := t;

    if t.flags & .SHARED {
        result = New(CType);
        result.* = t.*;
        result.flags &= ~CType.Flags.SHARED;
    }

    result.flags |= flags;

    return result;
}

modifiers_calling_convention :: (builder: *String_Builder, type: *Function_Type, assume_foreign: bool) {
    if type.flags & .OBJC_METHOD return;
    if type.flags & .CPP_METHOD append(builder, " #cpp_method");
    else if !assume_foreign append(builder, " #c_call");

    result := find_underlying_type(type.return_type);
    if !result || is_void_type(result) || !result.type_of_struct return;

    flags := result.type_of_struct.flags;
    non_pod: bool;

    if modifiers_options().os == .WINDOWS non_pod = (flags & .HAS_CONSTRUCTOR) != 0;
    else non_pod = (flags & (.HAS_MOVE_CONSTRUCTOR | .HAS_COPY_CONSTRUCTOR | .HAS_DESTRUCTOR)) != 0;
    if non_pod append(builder, " #cpp_return_type_is_non_pod");
}

modifiers_check_omission :: (func: *Function) {
    options := modifiers_options();
    flags := func.type.type_of_function.flags;
    reason: string;

    if options.strip_flags & .FUNCTIONS_WITH_VALIST && function_has_a_valist_arg(func) {
        reason = "va_list parameter";
    } else if options.strip_flags & .SYMBOLS_WITH_UNKNOWN_FOREIGN_LIBS && !(func.flags & (.VIRTUAL | .PURE_VIRTUAL)) && !(flags & .OBJC_METHOD) && function_is_missing_foreign_library(func) {
        reason = "missing foreign library";
    } else if options.strip_flags & .INLINED_FUNCTIONS && flags & .INLINE {
        reason = "inline function";
    }

    if !reason return;

    func.decl_flags |= .OMIT_FROM_OUTPUT;
    modifiers_generator().counts.stripped += 1;
    if options.log_stripped_declarations log("Bindings_Generator: omitted %: %\n", func.name, reason);
}
