#load "../../Bindings_Generator/declaration-emission.jai";
#load "binding-canonical-cpp-bridge.jai";

#scope_module
#import "Basic";

BE_Traversal :: #import "internal/developer-tools/emission-state";

binding_declaration :: (builder: *String_Builder, declaration: *Declaration, scope: *Declaration, flags: Print_Decl_Flags, alignment: int, existing: *[..] *Declaration, explicit_alignment: int) -> bool {
    generator := gp_current_generator();
    if !generator return false;

    conditional := declaration.size_condition_type != null;
    if conditional {
        append_indentation(builder);
        append(builder, "#if size_of(");
        print_type_to_builder(builder, declaration.size_condition_type, scope);
        append(builder, ") == 0 {\n");
        generator.tab_depth += 1;
    }

    defer if conditional generator.tab_depth -= 1;
    append_prefix_comment(builder, declaration.comment, false);
    if !(flags & .NO_INDENT) append_indentation(builder);

    if declaration.kind == .FUNCTION print_function(builder, cast(*Function) declaration, scope, declaration.foreign_name);
    else if declaration.kind == .ENUM print_enum(builder, cast(*Enum) declaration);
    else if declaration.kind == .TYPEDEF print_typedef(builder, cast(*Typedef) declaration);
    else if declaration.kind == .STRUCT print_struct(builder, cast(*Struct) declaration, existing, explicit_alignment);
    else if declaration.kind == .NAMESPACE print_namespace(builder, cast(*Namespace) declaration);
    else if declaration.kind == .MACRO_DEFINITION {
        if !declaration.expression || declaration.expression.kind != .LITERAL return false;

        literal := cast(*Literal) declaration.expression;

        if literal.literal_kind != .MACRO {
            // An evaluated constant standing in for an inexpressible spelling.
            if declaration.macro_value_result != .KEEP || !declaration.type return false;

            print_to_builder(builder, "% :: ", declaration.output_name);
            print_literal_to_builder(builder, literal, declaration.type, scope);
            append(builder, ";\n");
        } else {
            if declaration.macro_value_result == .NEEDS_CHECKING check_macro_value(declaration);
            if declaration.macro_value_result != .KEEP return false;

            text := string_from_macro_value(literal.macro_value, name = declaration.name);

            defer free(text);
            if !text return false;

            print_to_builder(builder, "% :: %;\n", declaration.output_name, text);
        }
    } else if declaration.kind == .BITFIELD {
        group := cast(*Bitfield) declaration;
        unwrapped := maybe_unwrap_bitfield_declaration(declaration);

        if unwrapped != declaration return binding_declaration(builder, unwrapped, scope, flags | .NO_INDENT, alignment, existing, explicit_alignment);

        if group.flags & .GENERATE_HELPERS && !array_find(generator.pending_bitfields, group) {
            session := context._bindings_generator_session;
            if session GM_Sessions.append_owned(session, *generator.pending_bitfields, group);
            else array_add(*generator.pending_bitfields, group);
        }

        if !binding_ordinary_declaration(builder, declaration, scope, flags, alignment, explicit_alignment) return false;
    } else if declaration.kind == .DECLARATION {
        if !binding_ordinary_declaration(builder, declaration, scope, flags, alignment, explicit_alignment) return false;
    } else {
        log_error("Bindings_Generator: unsupported declaration kind %\n", declaration.kind);
        return false;
    }

    if conditional {
        for 0..generator.tab_depth - 2 append(builder, TAB_STRING);
        append(builder, "}\n");
    }

    return !builder.failed;
}

binding_ordinary_declaration :: (builder: *String_Builder, declaration: *Declaration, scope: *Declaration, flags: Print_Decl_Flags, alignment: int, explicit_alignment: int) -> bool {
    if !declaration.type || !looks_like_valid_identifier(declaration.output_name) return false;

    type := declaration.type;

    if declaration.decl_flags & .POINTER_TO_VALUE && !(flags & .KEEP_POINTER_TO_VALUE) {
        type = find_underlying_type(type);
        if !type || !type.pointer_to return false;
        type = type.pointer_to;
    }

    if !binding_value_type_supported(type) {
        log_error("Bindings_Generator: incomplete native record cannot be emitted by value\n");
        return false;
    }

    if declaration.decl_flags & .QUALIFIER_USING append(builder, "using ");
    else if declaration.decl_flags & .QUALIFIER_AS append(builder, "#as ");

    append(builder, declaration.output_name);
    for 0..alignment - declaration.output_name.count - 1 append(builder, " ");

    initializer := declaration.expression != null && !(flags & .NO_ARGUMENT_DEFAULTS);
    inferred := initializer && can_omit_type(declaration, flags, type);
    constant := (declaration.decl_flags & .IS_CONST) != 0 && !(flags & .IN_FUNCTION_ARGS);

    if inferred append(builder, ifx constant then " :: " else " := ");
    else {
        append(builder, ": ");
        print_type_to_builder(builder, type, scope, flags, null);
        if initializer append(builder, ifx constant then " : " else " = ");
    }

    if initializer {
        value: String_Builder;

        defer free_buffers(*value);
        enum_printed := print_expression_to_builder(*value, declaration.expression, type, scope);

        if value.failed return false;

        text := builder_to_string(*value);

        defer free(text);

        if declaration.decl_flags & .NEEDS_DEFAULT_ARGUMENT_CAST && !enum_printed {
            append(builder, "cast(");
            print_type_to_builder(builder, type, scope, flags, null);
            print_to_builder(builder, ") (%)", text);
        } else append(builder, text);
    }

    if declaration.foreign_name && (!initializer || constant) {
        append(builder, " ");
        append_elsewhere(builder, declaration, declaration.output_name, declaration.foreign_name);
    }

    field_alignment := ifx explicit_alignment > 0 then explicit_alignment else declaration.explicit_field_alignment;
    // Basic's wide integers are limb records. Their Jai natural alignment can
    // differ from the SDK integer they represent, so retain that native field
    // alignment explicitly rather than relying on the replacement type layout.
    if field_alignment == 0 && declaration.field_alignment > 0 && scope && scope.kind == .STRUCT {
        base := strip_type(type, typedefs = true, enums = true);
        if binding_wide_integer_type(base) {
            natural := ifx base.number_flags & .SIGNED then align_of(S128) else align_of(U128);
            if declaration.field_alignment != natural field_alignment = declaration.field_alignment;
        }
    }

    if field_alignment > 0 print_to_builder(builder, " #align %", field_alignment);
    if declaration.bitfield_width > 0 print_to_builder(builder, " /* % bits */", declaration.bitfield_width);

    if !(flags & .NO_SEMICOLON) {
        append(builder, ";");
        append_postfix_comment(builder, declaration.comment);
        append(builder, "\n");
    }

    return !builder.failed;
}

binding_function :: (builder: *String_Builder, function: *Function, scope: *Declaration, foreign_name: string) -> bool {
    if !looks_like_valid_identifier(function.output_name) return false;

    type := function.type.type_of_function;
    if type.flags & (.OBJC_METHOD | .OBJC_CLASS_METHOD) {
        return binding_objc_function(builder, function, scope);
    }

    formatting := (function.decl_flags & .NEEDS_PRINTF_WRAPPER) != 0;
    wrapper := formatting || (function.decl_flags & .NEEDS_ARGUMENT_WRAPPER) != 0;

    if wrapper && !formatting && (type.flags & .VARIADIC) != 0 {
        log_error("Bindings_Generator: cannot automatically wrap non-formatting variadic arguments\n");
        return false;
    }

    foreign_label := function.output_name;
    if function.flags & .VIRTUAL foreign_label = tprint("virtual_%", foreign_label);
    if formatting foreign_label = tprint("%_CFormat", foreign_label);

    if !wrapper {
        print_to_builder(builder, "% :: ", foreign_label);
        print_function_type_to_builder(builder, type, scope, function);
        append(builder, " ");
        append_elsewhere(builder, cast(*Declaration) function, foreign_label, foreign_name);
        append(builder, ";\n");

        return !builder.failed;
    }

    options := gp_current_options();
    if !options || !options.get_func_args_for_printing return false;

    foreign_args, outer, inner: [..] *Declaration;

    defer array_free(foreign_args);
    defer array_free(outer);
    defer array_free(inner);
    // Callbacks can reuse their slice storage on the next invocation. Preserve
    // each selected pointer sequence before requesting another printing mode.
    if !binding_copy_argument_view(options.get_func_args_for_printing(function, type, .FOREIGN_FUNC_DECLARATION), *foreign_args) return false;
    if !binding_copy_argument_view(options.get_func_args_for_printing(function, type, .OUTER_WRAPPER), *outer) return false;
    if !binding_copy_argument_view(options.get_func_args_for_printing(function, type, .INNER_WRAPPER_CALL), *inner) return false;
    if formatting && (!outer.count || inner.count != foreign_args.count || inner.count < 1 || inner[inner.count - 1].output_name != outer[outer.count - 1].output_name) return false;

    foreign_type := type.*;
    foreign_type.arguments.count = foreign_args.count;
    foreign_type.arguments.data = foreign_args.data;
    transformed: [..] *Declaration;

    defer array_free(transformed);
    count_decl, data_decl: Declaration;
    data_type: CType;

    if formatting {
        for index: 0..foreign_args.count - 2 array_add(*transformed, foreign_args[index]);

        count_decl.kind = .DECLARATION;
        count_decl.output_name = "message_count";
        count_decl.type = gp_current_generator().type_def_s64;
        data_type.pointer_to = gp_current_generator().type_def_char;
        data_type.size = gp_current_generator().pointer_size;
        data_type.alignment = data_type.size;
        data_decl.kind = .DECLARATION;
        data_decl.output_name = "message_data";
        data_decl.type = *data_type;
        avoid_name_clash(*transformed, *count_decl, false);
        avoid_name_clash(*transformed, *data_decl, false);
        foreign_type.arguments.count = transformed.count;
        foreign_type.arguments.data = transformed.data;
        foreign_type.flags &= ~Function_Type.Flags.VARIADIC;
    }

    print_to_builder(builder, "% :: ", foreign_label);
    if !jai_model_function(builder, *foreign_type, scope, .NO_ARGUMENT_DEFAULTS | .KEEP_POINTER_TO_VALUE, 0) return false;

    append_calling_convention_directives(builder, *foreign_type, false);
    append(builder, " ");
    append_elsewhere(builder, cast(*Declaration) function, foreign_label, foreign_name);
    append(builder, ";\n");
    print_to_builder(builder, "% :: (", function.output_name);

    for argument, index: outer {
        if index append(builder, ", ");

        exposed := argument.*;
        underlying := find_underlying_type(exposed.type);

        if underlying && (underlying.flags & .REFERENCE) != 0 && underlying.pointer_to && (underlying.pointer_to.flags & .CONST) != 0 exposed.decl_flags |= .POINTER_TO_VALUE;

        if formatting && index == outer.count - 1 {
            exposed.type = gp_current_generator().type_def_jai_string;
            exposed.expression = null;
            exposed.decl_flags &= ~Declaration_Flags.POINTER_TO_VALUE;
        }

        if !binding_ordinary_declaration(builder, *exposed, scope, .IN_FUNCTION_ARGS | .NO_SEMICOLON | .NO_INDENT, 0, 0) return false;
    }

    if formatting append(builder, ", __args: ..Any");

    append(builder, ") -> ");
    print_type_to_builder(builder, type.return_type, scope, .IN_FUNCTION_RETURN);
    append(builder, " {\n");

    if formatting {
        print_to_builder(builder, " message := sprint(%, ..__args);\n defer free(message);\n", outer[outer.count - 1].output_name);
        gp_current_generator().generated_printf_wrapper = true;
    }

    if !is_void_type(type.return_type) append(builder, " return ");
    else append(builder, " ");

    print_to_builder(builder, "%(", foreign_label);
    limit := inner.count - ifx formatting then 1 else 0;
    for index: 0..limit - 1 {
        if index append(builder, ", ");

        argument := inner[index];
        named := false;

        for exposed: outer if exposed.output_name == argument.output_name named = true;
        if !named return false;

        underlying := find_underlying_type(argument.type);
        address := (argument.decl_flags & .POINTER_TO_VALUE) != 0 || (underlying && (underlying.flags & .REFERENCE) != 0 && underlying.pointer_to && (underlying.pointer_to.flags & .CONST) != 0);

        if address append(builder, "*");

        append(builder, argument.output_name);
    }

    if formatting {
        if limit append(builder, ", ");
        append(builder, "message.count, cast(");
        print_type_to_builder(builder, *data_type, scope);
        append(builder, ") message.data");
    }

    append(builder, ");\n}\n");

    return !builder.failed;
}

binding_objc_local_name :: (arguments: [] *Declaration, prefix: string) -> string {
    for index: 0..arguments.count {
        candidate := sprint("%_%", prefix, index);
        occupied := false;

        for argument: arguments if argument.output_name == candidate occupied = true;
        if !occupied return candidate;

        free(candidate);
    }

    assert(false, "Bindings_Generator: finite argument name set must leave a fresh local");

    return "";
}

binding_objc_function :: (builder: *String_Builder, function: *Function, scope: *Declaration) -> bool {
    signature := function.type.type_of_function;
    generator := gp_current_generator();

    if !generator || !signature.arguments.count || !binding_argument_view_valid(signature.arguments) {
        log_error("Bindings_Generator: Objective-C wrapper requires a represented receiver argument\n");
        return false;
    }

    if signature.flags & .VARIADIC {
        log_error("Bindings_Generator: variadic Objective-C wrapper is unsupported\n");
        return false;
    }

    runtime_name := binding_objc_local_name(signature.arguments, "__objc_runtime");
    send_name := binding_objc_local_name(signature.arguments, "__objc_send");
    call_name := binding_objc_local_name(signature.arguments, "__objc_call");
    selector_arg_name := binding_objc_local_name(signature.arguments, "__objc_selector");

    defer {
        free(runtime_name);
        free(send_name);
        free(call_name);
        free(selector_arg_name);
    }

    selector_name := tprint("SEL_%", function.output_name);
    selector := Declaration.{ kind = .DECLARATION, library = function.library };

    print_to_builder(builder, "%: *void ", selector_name);
    append_elsewhere(builder, *selector, selector_name, selector_name);
    append(builder, ";\n");
    outer := signature.*;
    outer.flags &= ~(Function_Type.Flags.OBJC_METHOD | .OBJC_CLASS_METHOD | .INLINE);
    print_to_builder(builder, "% :: ", function.output_name);
    if !jai_model_function(builder, *outer, scope, 0, 0) return false;

    print_to_builder(builder, " {\n % :: #library,system \"objc\";\n % :: () #foreign % ", runtime_name, send_name, runtime_name);
    result := strip_type(signature.return_type, typedefs = true);
    entry := "objc_msgSend";

    if result && result.type_of_struct entry = "objc_msgSend_stret";
    else if result && (result.number_flags & .FLOAT) != 0 entry = "objc_msgSend_fpret";

    emit_model_quoted(builder, entry);
    print_to_builder(builder, ";\n % := cast(", call_name);
    arguments: [..] *Declaration;

    defer array_free(arguments);
    array_add(*arguments, signature.arguments[0]);
    selector_arg := Declaration.{ output_name = selector_arg_name, type = generator.type_def_ptr_void };

    array_add(*arguments, *selector_arg);
    for index: 1..signature.arguments.count - 1 array_add(*arguments, signature.arguments[index]);

    inner := outer;
    inner.arguments.count = arguments.count;
    inner.arguments.data = arguments.data;
    if !jai_model_function(builder, *inner, scope, .NO_ARGUMENT_DEFAULTS, 0) return false;

    print_to_builder(builder, " #c_call) %;\n ", send_name);
    if !is_void_type(signature.return_type) append(builder, "return ");

    print_to_builder(builder, "%(%, %", call_name, signature.arguments[0].output_name, selector_name);
    for index: 1..signature.arguments.count - 1 print_to_builder(builder, ", %", signature.arguments[index].output_name);

    append(builder, ");\n}\n");

    return !builder.failed;
}

binding_argument_view_valid :: (arguments: [] *Declaration) -> bool {
    if arguments.count < 0 || (arguments.count && !arguments.data) return false;
    for argument: arguments if !argument || !argument.type || !looks_like_valid_identifier(argument.output_name) return false;
    return true;
}

binding_copy_argument_view :: (source: [] *Declaration, destination: *[..] *Declaration) -> bool {
    if !binding_argument_view_valid(source) return false;
    for argument: source array_add(destination, argument);
    return true;
}

binding_vtable_helpers :: (builder: *String_Builder, record: *Struct, info: Struct_Extra_Info) -> bool {
    options := gp_current_options();
    if !options || !record || !record.type || !info.vtable_struct return false;
    if !options.generate_c_style_api_for_vtable && !options.generate_vtable_helpers return true;

    pointer := info.vtable_pointer_declaration;
    if !pointer || !looks_like_valid_identifier(pointer.output_name) || !pointer.type return false;

    if options.generate_c_style_api_for_vtable for slot: info.vtable_struct.declarations {
        if !slot || !slot.type return false;

        signature := slot.type.type_of_function;
        if !signature continue; // Omitted slots retain storage, never a callable helper.

        if signature.flags & .VARIADIC {
            log_error("Bindings_Generator: variadic vtable slot wrapper is unsupported\n");
            continue;
        }

        if !signature.arguments.count || !binding_argument_view_valid(signature.arguments) return false;

        append_indentation(builder);
        print_to_builder(builder, "%_% :: inline ", record.output_name, slot.output_name);
        if !jai_model_function(builder, signature, cast(*Declaration) record, 0, 0) return false;

        append(builder, " { ");
        if !is_void_type(signature.return_type) append(builder, "return ");

        print_to_builder(builder, "%.%.%(", signature.arguments[0].output_name, pointer.output_name, slot.output_name);

        for argument, index: signature.arguments {
            if index > 0 append(builder, ", ");
            append(builder, argument.output_name);
        }

        append(builder, "); }\n");
    }

    if options.generate_vtable_helpers {
        append_indentation(builder);
        append(builder, "vtable :: inline (obj: *");
        if !jai_model_type(builder, record.type, cast(*Declaration) record, 0, 0) return false;

        append(builder, ") -> ");
        if !jai_model_type(builder, pointer.type, cast(*Declaration) record, 0, 0) return false;

        print_to_builder(builder, " { return obj.%; }\n", pointer.output_name);
    }

    return !builder.failed;
}

// A type name/typedef may identify an opaque record, but native value storage
// requires its layout. Actual pointers stop this walk before the pointee.
binding_value_type_supported :: (type: *CType) -> bool {
    seen: [..] *CType;

    defer array_free(seen);

    while type {
        if seen.count >= 128 || array_find(seen, type) return false;

        array_add(*seen, type);

        if type.type_of_typedef {
            type = type.type_of_typedef.type;
            continue;
        }

        if type.pointer_to return true;

        if type.array_element_type {
            type = type.array_element_type;
            continue;
        }

        if !type.type_of_struct return true;
        if type.size == cast(s64) CXTypeLayoutError.Incomplete || type.alignment == cast(s64) CXTypeLayoutError.Incomplete return false;

        canonical := type.type_of_struct.type;

        return !canonical || (canonical.size != cast(s64) CXTypeLayoutError.Incomplete && canonical.alignment != cast(s64) CXTypeLayoutError.Incomplete);
    }

    return false;
}

binding_native_incomplete_record :: (record: *Struct, explicit_alignment: int) -> bool {
    if !record || !record.type || record.type.type_of_struct != record || explicit_alignment != 0 return false;
    if record.type.size != cast(s64) CXTypeLayoutError.Incomplete || record.type.alignment != cast(s64) CXTypeLayoutError.Incomplete return false;
    if record.declarations.count != 0 || record.parents.count != 0 || record.virtual_functions.count != 0 || record.overlays.count != 0 || record.primary_field != null return false;
    if record.template_type_params.count != 0 || record.decl_flags & .TEMPLATE return false;
    if record.flags & (.IS_VTABLE | .IS_RAW | .OBJC_INTERFACE | .OBJC_PROTOCOL | .OBJC_CATEGORY | .HAS_CONSTRUCTOR | .HAS_COPY_CONSTRUCTOR | .HAS_MOVE_CONSTRUCTOR | .HAS_DESTRUCTOR) return false;

    generator := gp_current_generator();
    if !generator || !generator.translation_unit || !gm_current_session() return false;

    entries := generator.declarations_by_cursor.entries;
    if entries.count <= 0 || !entries.data return false;
    if entries.count > 0x7fffffffffffffff / size_of(type_of(entries[0])) return false;

    found := false;
    canonical: CXCursor;

    for entry: entries {
        if entry.hash < GM_Table.FIRST_VALID_HASH || entry.value != cast(*Declaration) record continue;

        cursor := entry.key;
        if cursor.kind != .StructDecl && cursor.kind != .UnionDecl && cursor.kind != .ClassDecl return false;
        if (cursor.kind == .UnionDecl) != ((record.flags & .IS_UNION) != 0) return false;
        if clang_Cursor_getTranslationUnit(cursor) != generator.translation_unit return false;
        if clang_isCursorDefinition(cursor) != 0 || clang_Cursor_isNull(clang_getCursorDefinition(cursor)) == 0 return false;

        native := clang_getCursorType(cursor);
        if clang_Type_getSizeOf(native) != cast(s64) CXTypeLayoutError.Incomplete || clang_Type_getAlignOf(native) != cast(s64) CXTypeLayoutError.Incomplete return false;

        key := clang_getCanonicalCursor(cursor);
        if found && clang_equalCursors(key, canonical) == 0 return false;

        canonical = key;
        found = true;
    }

    return found;
}

binding_anonymous_member :: (builder: *String_Builder, record: *Struct, existing: *[..] *Declaration, index: int) -> bool {
    append_indentation(builder);
    print_to_builder(builder, "using anonymous_%: ", index);
    if !binding_struct_body(builder, record, existing, 0) return false;

    append(builder, ";\n");

    return !builder.failed;
}

binding_struct_body :: (builder: *String_Builder, record: *Struct, existing: *[..] *Declaration, explicit_alignment: int) -> bool {
    generator := gp_current_generator();
    if !generator || !record || !record.type return false;

    if !BE_Traversal.enter(.DECLARATION_BODY, record) return false;

    defer BE_Traversal.leave();

    fields: [] *Declaration;
    alignment := record.type.alignment;
    extra_info: *Struct_Extra_Info;

    generic := record.template_type_params.count > 0;

    if generic {
        // A generic declaration has no single native layout to pad or validate.
        fields = record.declarations;
        alignment = 0;
    } else if record.flags & .IS_VTABLE fields = record.declarations;
    else {
        info, _generated := find_or_create_extra_info(record.type, cast(*Declaration) record);
        if info == *struct_extra_info_unknown && binding_native_incomplete_record(record, explicit_alignment) {
            append(builder, "struct");
            append_type_info_directives(builder, cast(*Declaration) record);
            append(builder, " {} // Native incomplete type; pointers only.");
            return !builder.failed;
        }

        if !info || info == *struct_extra_info_unknown {
            log_error("Bindings_Generator: record printing requires canonical supported layout metadata\n");
            return false;
        }

        copied := info.*;

        extra_info = info;

        fields = copied.modified_declarations;
        alignment = copied.struct_alignment;
    }

    append(builder, ifx record.flags & .IS_UNION then "union" else "struct");

    if generic {
        append(builder, "(");

        for parameter, index: record.template_type_params {
            if !parameter return false;
            if index > 0 append(builder, ", ");
            print_declaration_to_builder(builder, parameter, cast(*Declaration) record, .IN_FUNCTION_ARGS | .NO_INDENT | .NO_SEMICOLON);
            if builder.failed return false;
        }

        append(builder, ")");
    }

    append_type_info_directives(builder, cast(*Declaration) record);
    if explicit_alignment > 0 alignment = explicit_alignment;
    if alignment > 0 print_to_builder(builder, " #align %", alignment);

    append(builder, " {\n");
    generator.tab_depth += 1;
    defer generator.tab_depth -= 1;

    anonymous_members := 0;

    for field: fields {
        if !field return false;
        if field.decl_flags & .OMIT_FROM_OUTPUT continue;

        // An unnamed tag used by a containing field is emitted with that field's
        // type, rather than as a second declaration with no usable identifier.
        if field.kind == .STRUCT && (!field.output_name || (cast(*Struct) field).flags & .IS_ANON_INLINE_TYPE) {
            represented := false;
            for member: fields if member && member.kind == .DECLARATION && try_find_struct_in_type(member.type) == cast(*Struct) field represented = true;
            if represented continue;

            // A C11 anonymous member (`union { ... };` inside a record) has no field of its
            // own. It becomes a `using` member so that its fields stay reachable directly.
            if !field.output_name {
                if !binding_anonymous_member(builder, cast(*Struct) field, existing, anonymous_members) return false;

                anonymous_members += 1;
                continue;
            }
        }

        print_declaration_to_builder(builder, field, cast(*Declaration) record, existing_declarations = existing);
        if builder.failed return false;
    }

    if extra_info && extra_info.vtable_struct {
        for 0..generator.tab_depth - 1 append(builder, TAB_STRING);
        print_struct_vtable(builder, record, extra_info.*);
        if builder.failed return false;
    }

    for 0..generator.tab_depth - 2 append(builder, TAB_STRING);

    append(builder, "}");

    return !builder.failed;
}
