// Shared model adapter emission: native cursor signatures retain overload identity.
#scope_module
#import "Basic";

custom_model_names_valid :: (model: *Native_Model) -> bool {
    names: [..] string;

    defer array_free(names);

    for declaration: model.root.declarations {
        if declaration.decl_flags & .OMIT_FROM_OUTPUT continue;
        if declaration.kind == .MACRO_DEFINITION && declaration.macro_value_result == .DISCARD continue;
        if declaration.kind == .TYPEDEF && declaration.type && declaration.type.type_of_struct && declaration.output_name == declaration.type.type_of_struct.output_name continue;

        if !looks_like_valid_identifier(declaration.output_name) {
            log_error("Bindings_Generator: invalid output declaration name %\n", declaration.output_name);
            return false;
        }

        if array_find(names, declaration.output_name) {
            log_error("Bindings_Generator: output declaration name collision %\n", declaration.output_name);
            return false;
        }

        array_add(*names, declaration.output_name);
    }

    return true;
}

custom_cpp_base :: (type: *CType) -> *CType {
    seen: [..] *CType;

    defer array_free(seen);

    while type && type.type_of_typedef {
        if array_find(seen, type) return null;
        array_add(*seen, type);
        type = type.type_of_typedef.type;
    }

    return type;
}

custom_cpp_declarations_valid :: (model: *Native_Model) -> bool {
    for declaration, index: model.cpp_functions {
        if declaration.decl_flags & .OMIT_FROM_OUTPUT || !array_find(model.root.declarations, declaration) continue;

        cursor := model.cpp_cursors[index];

        if clang_getCursorLinkage(cursor) != .External || clang_getCursorVisibility(cursor) == .Hidden || clang_getCursorAvailability(cursor) != .Available return false;
    }

    return true;
}

custom_cpp_boundary_type :: (builder: *String_Builder, type: *CType) -> bool {
    base := custom_cpp_base(type);
    if !base || base.hardcoded_jai_string || base.array_element_type || base.type_of_function return false;
    if base.pointer_to && (base.pointer_to.array_element_type || base.pointer_to.type_of_function) return false;

    if base.flags & .REFERENCE {
        pointer := base.*;
        pointer.flags &= ~CType.Flags.REFERENCE;
        return cpp_type(builder, *pointer, null);
    }

    return cpp_type(builder, type, null);
}

native_cpp_array_qualifiers :: (type: *CType) -> CType.Flags, bool {
    if !type || !BE_Traversal.enter(.CPP_ARRAY_QUALIFIERS, type) return 0, false;

    defer BE_Traversal.leave();

    qualifiers := type.flags & (CType.Flags.CONST | .VOLATILE);
    next: *CType;

    if type.type_of_typedef next = type.type_of_typedef.type;
    else if type.array_element_type next = type.array_element_type;

    if next {
        nested, valid := native_cpp_array_qualifiers(next);
        return qualifiers | nested, valid;
    }

    return qualifiers, true;
}

native_cpp_shape_matches :: (type: *CType, native: CXType, depth := 0, inherited_qualifiers: CType.Flags = 0) -> bool {
    if !type || !BE_Traversal.enter(.CPP_MATCH, type) return false;

    defer BE_Traversal.leave();

    base := custom_cpp_base(type);
    if !base || base.hardcoded_jai_string || base.type_of_struct || base.type_of_enum || base.type_of_declaration return false;

    view := base.*;
    view.flags |= type.flags & (CType.Flags.CONST | .VOLATILE);
    native = clang_getCanonicalType(native);

    native_qualifiers := inherited_qualifiers;
    if clang_isConstQualifiedType(native) != 0 native_qualifiers |= .CONST;
    if clang_isVolatileQualifiedType(native) != 0 native_qualifiers |= .VOLATILE;

    model_qualifiers := view.flags & (CType.Flags.CONST | .VOLATILE);

    if view.array_element_type {
        nested, valid := native_cpp_array_qualifiers(*view);
        if !valid return false;
        model_qualifiers |= nested;
    }

    if depth > 0 {
        if model_qualifiers != native_qualifiers return false;
    }

    if view.pointer_to {
        if native.kind != .Pointer && native.kind != .LValueReference return false;
        return native_cpp_shape_matches(view.pointer_to, clang_getPointeeType(native), depth + 1);
    }

    if view.array_element_type {
        if native.kind != .ConstantArray && native.kind != .IncompleteArray return false;
        if view.array_element_count != clang_getArraySize(native) return false;
        return native_cpp_shape_matches(view.array_element_type, clang_getArrayElementType(native), depth + 1, native_qualifiers);
    }

    if view.type_of_function {
        if native.kind != .FunctionProto || clang_getFunctionTypeCallingConv(native) != .C return false;

        function := view.type_of_function;
        if function.flags & (.CPP_METHOD | .CONSTRUCTOR | .DESTRUCTOR | .OBJC_METHOD | .OBJC_CLASS_METHOD) return false;
        if function.arguments.count != clang_getNumArgTypes(native) return false;
        if ((function.flags & .VARIADIC) != 0) != (clang_isFunctionTypeVariadic(native) != 0) return false;
        if !native_cpp_shape_matches(function.return_type, clang_getResultType(native), depth + 1) return false;

        for argument, index: function.arguments {
            if !argument || !native_cpp_shape_matches(argument.type, clang_getArgType(native, cast(u32) index), depth + 1) return false;
        }

        return true;
    }

    if view.flags & .VOID return native.kind == .Void;
    if view.flags & .BOOL return native.kind == .Bool;
    if !view.number_flags || view.size != clang_Type_getSizeOf(native) return false;

    signed := native.kind == .Char_S || native.kind == .SChar || native.kind == .Short || native.kind == .Int || native.kind == .Long || native.kind == .LongLong;
    unsigned := native.kind == .Char_U || native.kind == .UChar || native.kind == .UShort || native.kind == .UInt || native.kind == .ULong || native.kind == .ULongLong;
    floating := native.kind == .Float || native.kind == .Double;

    if view.number_flags & .FLOAT return floating;
    if floating || (!signed && !unsigned) return false;

    return ((view.number_flags & .SIGNED) != 0) == signed;
}

custom_cpp_native_boundary :: (builder: *String_Builder, type: *CType, native: CXType, policy: CXPrintingPolicy) -> bool {
    if !native_cpp_shape_matches(type, native) {
        valid := custom_cpp_boundary_type(builder, type);
        if !valid log_error("Bindings_Generator: unsupported native C++ boundary type category %\n", native.kind);
        return valid;
    }

    canonical := clang_getCanonicalType(native);
    if canonical.kind == .LValueReference {
        append(builder, "::independently_generated_custom_bridge::InteropPointer<");
        append_native_cpp_type(builder, clang_getPointeeType(canonical), policy);
        append(builder, ">");
    } else {
        append(builder, "::independently_generated_custom_bridge::InteropType<");
        append_native_cpp_type(builder, native, policy);
        append(builder, ">");
    }

    return !builder.failed;
}

custom_cpp_arguments_valid :: (arguments: [] *Declaration) -> bool {
    for argument, index: arguments {
        if !argument || !argument.type || !looks_like_valid_identifier(argument.output_name) return false;
        for other: 0..index - 1 if arguments[other].output_name == argument.output_name return false;
    }

    return true;
}

custom_cpp_out_name :: (arguments: [] *Declaration) -> string {
    for suffix: 0..arguments.count - 1 {
        name := tprint("out_result_%", suffix);
        found := false;
        for argument: arguments if argument.output_name == name found = true;
        if !found return name;
    }

    return tprint("out_result_%", arguments.count);
}

emit_custom_cpp_jai :: (builder: *String_Builder, declaration: *Function, get_arguments: Get_Function_Arguments_For_Printing_Func) -> bool {
    if !declaration.type || !declaration.type.type_of_function return false;

    function := declaration.type.type_of_function;
    if function.flags & .VARIADIC return false;

    result := custom_cpp_base(function.return_type);
    if !result return false;

    aggregate := result.type_of_struct != null;
    arguments := get_arguments(declaration, function, .FOREIGN_FUNC_DECLARATION);

    if !custom_cpp_arguments_valid(arguments) return false;

    append(builder, "(");

    if aggregate {
        print_to_builder(builder, "%: *", custom_cpp_out_name(arguments));
        if !emit_model_type(builder, function.return_type) return false;
    }

    for argument, index: arguments {
        if aggregate || index > 0 append(builder, ", ");
        print_to_builder(builder, "%: ", argument.output_name);
        if !emit_model_type(builder, argument.type) return false;
    }

    append(builder, ")");

    if !aggregate && !(result.flags & .VOID) {
        append(builder, " -> ");
        if !emit_model_type(builder, function.return_type) return false;
    }

    return true;
}

emit_custom_cpp_bridge :: (builder: *String_Builder, model: *Native_Model, options: Generate_Bindings_Options) -> bool {
    if !custom_model_names_valid(model) || !custom_cpp_declarations_valid(model) return false;

    append(builder, "namespace independently_generated_custom_bridge { template<class T> using InteropPointer = T*; template<class T> using InteropType = T; }\n");

    for declaration, index: model.cpp_functions {
        if declaration.decl_flags & .OMIT_FROM_OUTPUT || !array_find(model.root.declarations, declaration) continue;
        if !cpp_identifier(declaration.output_name) return false;

        cursor := model.cpp_cursors[index];
        policy := clang_getCursorPrintingPolicy(cursor);

        if !policy return false;

        defer clang_PrintingPolicy_dispose(policy);
        clang_PrintingPolicy_setProperty(policy, .SuppressScope, 0);
        clang_PrintingPolicy_setProperty(policy, .FullyQualifiedName, 1);

        if !declaration.type || !declaration.type.type_of_function return false;

        function := declaration.type.type_of_function;
        if function.flags & .VARIADIC return false;

        result := custom_cpp_base(function.return_type);
        if !result return false;

        aggregate := result.type_of_struct != null;
        reference := (result.flags & .REFERENCE) != 0;
        nothing := (result.flags & .VOID) != 0;
        foreign := options.get_func_args_for_printing(cast(*Function) declaration, function, .FOREIGN_FUNC_DECLARATION);
        // Callbacks may reuse their own array storage for another mode.
        parameters: [..] *Declaration;

        defer array_free(parameters);
        for parameter: foreign array_add(*parameters, parameter);
        if !custom_cpp_arguments_valid(parameters) return false;

        inner := options.get_func_args_for_printing(cast(*Function) declaration, function, .INNER_WRAPPER_CALL);
        native := clang_getCursorType(cursor);
        count := clang_getNumArgTypes(native);

        if count < 0 || inner.count != count || clang_isFunctionTypeVariadic(native) != 0 || clang_getFunctionTypeCallingConv(native) != .C return false;

        native_result := clang_getResultType(native);
        canonical_result := clang_getCanonicalType(native_result);

        if canonical_result.kind == .Record && clang_isPODType(canonical_result) == 0 return false;
        if aggregate != (canonical_result.kind == .Record) || reference != (canonical_result.kind == .LValueReference) || nothing != (canonical_result.kind == .Void) return false;

        append(builder, "namespace independently_generated_custom_bridge {\nextern \"C\" ");
        if aggregate append(builder, "void");
        else if !custom_cpp_native_boundary(builder, function.return_type, native_result, policy) return false;

        print_to_builder(builder, " %(", declaration.output_name);
        out_name := custom_cpp_out_name(parameters);

        if aggregate {
            if !cpp_type(builder, result, null, omit_top_cv = true) return false;
            print_to_builder(builder, " *%", out_name);
        }

        for parameter, parameter_index: parameters {
            if aggregate || parameter_index > 0 append(builder, ", ");
            if !cpp_identifier(parameter.output_name) return false;

            native_index: s64 = -1;
            for argument, index: inner if argument == parameter && native_index < 0 native_index = index;

            if native_index >= 0 {
                if !custom_cpp_native_boundary(builder, parameter.type, clang_getArgType(native, cast(u32) native_index), policy) return false;
            } else if !custom_cpp_boundary_type(builder, parameter.type) return false;

            print_to_builder(builder, " %", parameter.output_name);
        }

        append(builder, ") {\n  ");
        if aggregate print_to_builder(builder, "*% = ", out_name);
        else if !nothing append(builder, ifx reference then "return &(" else "return ");

        append(builder, "(static_cast<::independently_generated_custom_bridge::InteropPointer<");
        append_native_cpp_type(builder, native, policy);
        append(builder, ">>(&");
        if !append_native_cpp_name(builder, cursor) return false;

        append(builder, "))(");

        for argument, argument_index: inner {
            if !argument || !array_find(parameters, argument) return false;
            if argument_index > 0 append(builder, ", ");

            native_argument := clang_getArgType(native, cast(u32) argument_index);
            canonical := clang_getCanonicalType(native_argument);
            base := custom_cpp_base(argument.type);

            if !base return false;

            dereference := canonical.kind == .LValueReference;
            if dereference && !base.pointer_to return false;
            if canonical.kind == .Record && clang_isPODType(canonical) == 0 return false;

            append(builder, "static_cast<");
            append_native_cpp_type(builder, native_argument, policy);
            print_to_builder(builder, ">(%%)", ifx dereference then "*" else "", argument.output_name);
        }

        append(builder, ")");
        if reference append(builder, ")");

        append(builder, ";\n}\n}\n");
    }

    return !builder.failed;
}

append_native_cpp_type :: (builder: *String_Builder, type: CXType, policy: CXPrintingPolicy) {
    spelling := clang_getFullyQualifiedName(type, policy, 1);

    defer clang_disposeString(spelling);
    text := to_string(clang_getCString(spelling));
    if !text {
        log_error("Bindings_Generator: could not format qualified native C++ type\n");
        builder.failed = true;
        return;
    }

    append(builder, text);
}

append_native_cpp_name :: (builder: *String_Builder, cursor: CXCursor) -> bool {
    names: [..] string;

    defer {
        for name: names free(name);
        array_free(names);
    }

    for depth: 0..64 {
        if clang_Cursor_isNull(cursor) != 0 return false;

        if cursor.kind == .TranslationUnit {
            append(builder, "::");

            for index: 0..names.count - 1 {
                if index > 0 append(builder, "::");
                append(builder, names[names.count - 1 - index]);
            }

            return true;
        }

        if cursor.kind != .Namespace && cursor.kind != .FunctionDecl return false;

        spelling := clang_getCursorSpelling(cursor);
        name := copy_string(to_string(clang_getCString(spelling)));

        clang_disposeString(spelling);
        array_add(*names, name);
        if !cpp_identifier(name) return false;

        cursor = clang_getCursorSemanticParent(cursor);
    }

    return false;
}
