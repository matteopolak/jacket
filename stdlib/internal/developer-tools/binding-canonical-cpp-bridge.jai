#scope_module
#import "Basic";

binding_cpp_parameter_name :: (used: *[..] string, name: string) -> string {
    base := avoid_cpp_keywords(name);
    if !cpp_identifier(base) return "";

    result := copy_string(base);
    suffix := 1;

    while array_find(used.*, result) {
        free(result);
        result = sprint("%_%", base, suffix);
        suffix += 1;
    }

    array_add(used, result);

    return result;
}

binding_cpp_boundary :: (builder: *String_Builder, type: *CType, native: CXType, policy: CXPrintingPolicy, has_native: bool, pointer_alias: string) -> bool {
    base := strip_type(type, typedefs = true);
    if !base return false;

    matching := has_native && native_cpp_shape_matches(type, native);
    if has_native && base.pointer_to && base.flags & .INSERTED matching = native_cpp_shape_matches(base.pointer_to, native);

    if matching {
        if base.flags & .REFERENCE {
            print_to_builder(builder, "%<", pointer_alias);
            append_native_cpp_type(builder, clang_getPointeeType(native), policy);
            append(builder, ">");
        } else append_native_cpp_type(builder, native, policy);

        return !builder.failed;
    }

    // Native metadata refines matching shapes; public transforms remain the
    // authority when a caller replaces a represented boundary type.
    if base.flags & .REFERENCE {
        pointer := base.*;
        pointer.flags &= ~CType.Flags.REFERENCE;
        return cpp_declarator(builder, *pointer, null, "", 0, 0, false, false);
    }

    if base.pointer_to && base.flags & .INSERTED return cpp_declarator(builder, base.pointer_to, null, "", 0, 0, false, false);

    return cpp_declarator(builder, type, null, "", 0, 0, false, false);
}

binding_cpp_bridge :: (builder: *String_Builder, function: *Function) -> bool {
    if !cpp_identifier(function.output_name) return false;

    type := function.type.type_of_function;
    result := strip_type(type.return_type, typedefs = true);

    if !result return false;

    formatting := (function.decl_flags & .NEEDS_PRINTF_WRAPPER) != 0;
    if (type.flags & .VARIADIC) != 0 && !formatting {
        log_error("Bindings_Generator: non-formatting variadic C++ bridge cannot forward arbitrary arguments\n");
        return false;
    }

    if type.flags & (.OBJC_METHOD | .OBJC_CLASS_METHOD) return false;

    cursor := gm_declaration_cursor(cast(*Declaration) function);
    native := clang_getCursorType(cursor);
    has_native := clang_Cursor_isNull(cursor) == 0;
    policy: CXPrintingPolicy;

    if has_native {
        policy = clang_getCursorPrintingPolicy(cursor);
        if !policy return false;
        clang_PrintingPolicy_setProperty(policy, .SuppressScope, 0);
        clang_PrintingPolicy_setProperty(policy, .FullyQualifiedName, 1);
    }

    defer if policy clang_PrintingPolicy_dispose(policy);

    count := type.arguments.count;
    if count < 0 || (count && !type.arguments.data) return false;

    receiver := false;

    if has_native {
        native_count := clang_getNumArgTypes(native);
        if native_count < 0 return false;
        receiver = count == native_count + 1 && (type.flags & .CPP_METHOD) != 0;
        if count != native_count + cast(int) receiver return false;
    } else receiver = (type.flags & .CPP_METHOD) != 0;

    if receiver && !count return false;
    if (type.flags & (.CONSTRUCTOR | .DESTRUCTOR)) != 0 && !receiver return false;

    start := cast(int) receiver;
    limit := count - cast(int) formatting;

    if limit < start return false;

    aggregate := result.type_of_struct != null && !result.pointer_to && !(result.flags & .REFERENCE);
    reference := (result.flags & .REFERENCE) != 0;
    nothing := (result.flags & .VOID) != 0;

    names: [..] string;
    used: [..] string;

    defer array_free(names);

    defer {
        for name: used free(name);
        array_free(used);
    }

    for argument, index: type.arguments {
        if !argument || !argument.type return false;
        if index >= limit break;

        name := binding_cpp_parameter_name(*used, argument.output_name);
        if !name return false;

        array_add(*names, name);
    }

    out_name: string;
    count_name: string;
    data_name: string;

    if aggregate out_name = binding_cpp_parameter_name(*used, "out_return_value");

    if formatting {
        count_name = binding_cpp_parameter_name(*used, "message_count");
        data_name = binding_cpp_parameter_name(*used, "message_data");
    }

    // Local aliases preserve native long/long long and complex declarators without
    // a second type registry. Cursor-free callers use their represented CType.
    stem := tprint("Binding_%", function.output_name);
    pointer_alias := tprint("%_Pointer", stem);

    print_to_builder(builder, "template<class T> using % = T*;\n", pointer_alias);

    print_to_builder(builder, "using %_Result = ", stem);

    native_result: CXType;
    if has_native native_result = clang_getResultType(native);
    if !binding_cpp_boundary(builder, type.return_type, native_result, policy, has_native, pointer_alias) return false;

    append(builder, ";\n");

    for argument, index: type.arguments {
        if index >= limit break;

        print_to_builder(builder, "using %_Arg% = ", stem, index);

        if has_native && receiver && index == 0 {
            if clang_CXXMethod_isConst(cursor) != 0 append(builder, "const ");
            append_native_cpp_type(builder, clang_getCursorType(clang_getCursorSemanticParent(cursor)), policy);
            append(builder, " *");
        } else if has_native && index >= start {
            parameter := clang_getArgType(clang_getCanonicalType(native), cast(u32) (index - start));
            if !binding_cpp_boundary(builder, argument.type, parameter, policy, true, pointer_alias) return false;
        } else if !binding_cpp_boundary(builder, argument.type, .{}, null, false, pointer_alias) return false;

        append(builder, ";\n");
    }

    if has_native && !receiver && !(type.flags & .CPP_METHOD) && !formatting {
        print_to_builder(builder, "using %_Signature = ", stem);
        append_native_cpp_type(builder, native, policy);
        append(builder, ";\n");
    }

    append(builder, "namespace independently_generated_bindings_bridge {\n");
    print_to_builder(builder, "extern \"C\" % %(", ifx aggregate then "void" else tprint("%_Result", stem), function.output_name);
    if aggregate print_to_builder(builder, "%_Result *%", stem, out_name);

    defaults := !formatting;
    seen_default := false;

    for index: start..limit - 1 {
        if type.arguments[index].expression seen_default = true;
        else if seen_default defaults = false;
    }

    for argument, index: type.arguments {
        if index >= limit break;
        if aggregate || index append(builder, ", ");

        print_to_builder(builder, "%_Arg% %", stem, index, names[index]);

        if defaults && argument.expression {
            append(builder, " = ");

            if !append_cpp_expression(builder, argument.expression, current_scope = function.parent) {
                log_error("Bindings_Generator: unsupported native C++ default expression for %\n", argument.output_name);
                return false;
            }
        }
    }

    if formatting {
        if aggregate || limit append(builder, ", ");
        print_to_builder(builder, "long long %, const char *%", count_name, data_name);
    }

    append(builder, ") {\n  ");
    if aggregate print_to_builder(builder, "new (static_cast<void *>(%)) %_Result(", out_name, stem);
    else if !nothing append(builder, ifx reference then "return &(" else "return ");

    constructor := (type.flags & .CONSTRUCTOR) != 0;
    destructor := (type.flags & .DESTRUCTOR) != 0;

    if constructor {
        print_to_builder(builder, "new (static_cast<void *>(%)) ", names[0]);
        if !function.parent || !cpp_name(builder, function.parent, null) return false;
    } else if receiver {
        print_to_builder(builder, "%->", names[0]);

        if destructor {
            if !function.parent || !cpp_identifier(function.parent.name) return false;
            print_to_builder(builder, "~%", function.parent.name);
        } else {
            if !cpp_identifier(function.name) return false;
            append(builder, function.name);
        }
    } else {
        exact := has_native && !(type.flags & .CPP_METHOD) && !formatting;
        if exact print_to_builder(builder, "(static_cast<%_Signature *>(&", stem);
        if !cpp_name(builder, cast(*Declaration) function, null) return false;
        if exact append(builder, "))");
    }

    append(builder, "(");

    for index: start..limit - 1 {
        if index > start append(builder, ", ");

        argument := type.arguments[index];
        base := strip_type(argument.type, typedefs = true);

        if !base return false;

        if has_native {
            parameter := clang_getArgType(clang_getCanonicalType(native), cast(u32) (index - start));

            append(builder, "static_cast<");
            canonical := clang_getCanonicalType(parameter);
            if canonical.kind == .ConstantArray || canonical.kind == .IncompleteArray || canonical.kind == .DependentSizedArray print_to_builder(builder, "decltype(%)", names[index]);
            else append_native_cpp_type(builder, parameter, policy);

            print_to_builder(builder, ">(%%)", ifx canonical.kind == .LValueReference then "*" else "", names[index]);
        } else print_to_builder(builder, "%%", ifx base.flags & .REFERENCE then "*" else "", names[index]);
    }

    if formatting {
        if limit > start append(builder, ", ");
        append(builder, "\"%.*s\", ");
        print_to_builder(builder, "static_cast<int>(%), %", count_name, data_name);
    }

    append(builder, ")");
    if reference || aggregate append(builder, ")");

    append(builder, ";\n}\n}\n");
    // C-linkage exports keep their required names. The private C++ scope avoids
    // redeclaring an identically named global C++ function with C linkage.
    if function.parent || function.name != function.output_name print_to_builder(builder, "using independently_generated_bindings_bridge::%;\n", function.output_name);

    return !builder.failed;
}
