#load "binding-bitfields.jai";
#load "binding-declaration-emission.jai";

append_template_instantiation_params :: (builder: *String_Builder, params: [] *Expression, type_params: [] *Declaration, current_scope: *Declaration) {
    if params.count == 0 return;

    fragment: String_Builder;

    defer free_buffers(*fragment);
    valid := binding_template_arguments(*fragment, params, type_params, current_scope, 0);

    commit_type_fragment(builder, *fragment, valid);
}

print_typedef :: (builder: *String_Builder, def: *Typedef) {
    fragment: String_Builder;

    defer free_buffers(*fragment);
    valid := def != null;
    if valid valid = looks_like_valid_identifier(def.output_name) && (def.flags & (.ISA | .DISTINCT)) != (.ISA | .DISTINCT);

    if valid {
        print_to_builder(*fragment, "% :: ", def.output_name);
        if def.flags & .ISA append(*fragment, "#type,isa ");
        if def.flags & .DISTINCT append(*fragment, "#type,distinct ");

        valid = jai_model_type(*fragment, def.type, def.parent, 0, 0);
        append(*fragment, ";\n");
    }

    commit_type_fragment(builder, *fragment, valid);
}

print_enum :: (builder: *String_Builder, _enum: *Enum) {
    fragment: String_Builder;

    defer free_buffers(*fragment);
    options := gp_current_options();
    valid := _enum != null && options != null;

    if valid valid = looks_like_valid_identifier(_enum.output_name);

    if valid {
        if options.c_enum_emulation append(*fragment, "using ");
        print_to_builder(*fragment, "% :: ", _enum.output_name);
        valid = binding_enum_body(*fragment, _enum);
        append(*fragment, "\n");
    }

    commit_type_fragment(builder, *fragment, valid);
    if valid && builder && !builder.failed gp_current_generator().counts.enums += 1;
}

print_enum_body :: (builder: *String_Builder, _enum: *Enum) {
    fragment: String_Builder;

    defer free_buffers(*fragment);
    valid := binding_enum_body(*fragment, _enum);

    commit_type_fragment(builder, *fragment, valid);
    if valid && builder && !builder.failed gp_current_generator().counts.enums += 1;
}

append_cpp_type :: (builder: *String_Builder, ty: *CType, current_scope: *Declaration, flags: Print_Decl_Flags = 0) {
    fragment: String_Builder;
    defer free_buffers(*fragment);
    commit_type_fragment(builder, *fragment, cpp_declarator(*fragment, ty, current_scope, "", flags, 0, false, true));
}

append_cpp_type_modifiers :: (builder: *String_Builder, ty: *CType, flags: Print_Decl_Flags) {
    if !ty {
        fail_type_emission(builder);
        return;
    }

    if flags & .C_TYPE_DETAILS && ty.flags & .CONST append(builder, " const");
    if ty.flags & .REFERENCE append(builder, " &");
}

print_type_to_builder :: (builder: *String_Builder, ty: *CType, current_scope: *Declaration, flags: Print_Decl_Flags = 0, existing_declarations: *[..] *Declaration = null) {
    fragment: String_Builder;
    defer free_buffers(*fragment);
    commit_type_fragment(builder, *fragment, jai_model_type(*fragment, ty, current_scope, flags, 0, existing_declarations));
}

type_to_string :: (ty: *CType, current_scope: *Declaration, flags: Print_Decl_Flags = 0) -> string {
    builder: String_Builder;

    defer free_buffers(*builder);
    print_type_to_builder(*builder, ty, current_scope, flags);
    if builder.failed return "";

    return builder_to_string(*builder);
}

print_function_type_to_builder :: (builder: *String_Builder, type: *Function_Type, current_scope: *Declaration, func: *Function = null, flags: Print_Decl_Flags = 0) {
    fragment: String_Builder;

    defer free_buffers(*fragment);
    valid := type != null;
    view: Function_Type;

    if valid view = type.*;

    if valid && func {
        options := gp_current_options();

        valid = options != null && options.get_func_args_for_printing != null;
        if valid {
            selected := options.get_func_args_for_printing(func, type, .FOREIGN_FUNC_DECLARATION);
            valid = selected.count >= 0 && (selected.count == 0 || selected.data != null);
            view.arguments.count = selected.count;
            view.arguments.data = selected.data;
        }
    }

    if valid valid = jai_model_function(*fragment, *view, current_scope, flags, 0);
    if valid && !binding_valist_is_opaque(*view) append_calling_convention_directives(*fragment, *view, false);

    commit_type_fragment(builder, *fragment, valid);
}

exceeds_signed_int_value :: (value: u64, target_type: *CType) -> bool {
    type := strip_type(target_type, typedefs = true, enums = true);
    bytes: s64 = 8;

    if type bytes = type.size;

    // Every input value is u64, so it fits the represented signed128-bit type.
    if bytes == 16 return false;

    assert(bytes == 1 || bytes == 2 || bytes == 4 || bytes == 8, "unsupported signed integer width");
    maximum := (cast(u64) 1 << (bytes * 8 - 1)) - 1;

    return value > maximum;
}

strip_type :: (type: *CType, typedefs := false, enums := false, pointers := false) -> *CType {
    seen: [..] *CType;

    defer array_free(seen);

    while type {
        next: *CType;
        if typedefs && type.type_of_typedef next = type.type_of_typedef.type;
        else if enums && type.type_of_enum next = type.type_of_enum.type;
        else if pointers && type.pointer_to next = type.pointer_to;
        else return type;

        if array_find(seen, type) {
            log_error("Bindings_Generator: cyclic type stripping model\n");
            return null;
        }

        array_add(*seen, type);
        type = next;
    }

    return null;
}

#scope_module
#import "Basic";

binding_template_arguments :: (builder: *String_Builder, params: [] *Expression, formals: [] *Declaration, scope: *Declaration, depth: int) -> bool {
    if params.count < 0 || (params.count && !params.data) || formals.count < 0 || (formals.count && !formals.data) return false;
    if !params.count return true;
    if !BE_Traversal.enter(.TEMPLATE_ARGUMENTS, params.data) return false;

    defer BE_Traversal.leave();

    append(builder, "(");

    for expression, index: params {
        if index append(builder, ", ");

        target: *CType;
        if index < formals.count && formals[index] target = formals[index].type;

        enum_printed := false;
        if !jai_target_expression_fragment(builder, expression, target, scope, false, *enum_printed, depth + 1) return false;
    }

    append(builder, ")");

    return !builder.failed;
}

binding_enum_body :: (builder: *String_Builder, enumeration: *Enum) -> bool {
    generator := gp_current_generator();
    if !enumeration || !generator return false;

    integer := strip_type(enumeration.type, typedefs = true, enums = true);
    if enumeration.type && (!integer || !integer.number_flags || integer.number_flags & .FLOAT || integer.pointer_to) return false;

    append(builder, ifx enumeration.flags & .IS_ENUM_FLAGS then "enum_flags" else "enum");

    if enumeration.type {
        append(builder, " ");
        if !jai_model_type(builder, enumeration.type, enumeration.parent, 0, 0) return false;
    }

    append(builder, " {\n");
    generator.tab_depth += 1;
    defer generator.tab_depth -= 1;
    names: [..] string;

    defer array_free(names);

    for value: enumeration.enumerates {
        if !value return false;
        if value.decl_flags & .OMIT_FROM_OUTPUT || !value.output_name continue;
        if !looks_like_valid_identifier(value.output_name) || array_find(names, value.output_name) return false;

        array_add(*names, value.output_name);
        append_indentation(builder);
        print_to_builder(builder, "% :: ", value.output_name);
        enum_printed := false;

        if !value.expression return false;

        if value.expression.kind == .LITERAL {
            if !jai_literal_fragment(builder, cast(*Literal) value.expression, enumeration.type, cast(*Declaration) enumeration, (enumeration.flags & .VALUES_IN_HEX) != 0, *enum_printed) return false;
        } else if !jai_expression_fragment(builder, value.expression, cast(*Declaration) enumeration, (enumeration.flags & .VALUES_IN_HEX) != 0, *enum_printed, 0) return false;

        append(builder, ";");
        append_postfix_comment(builder, value.comment);
        append(builder, "\n");

        if enumeration.flags & .ALIAS_ORIGINAL_NAMES && value.name && value.name != value.output_name {
            reserved := false;
            for name: RESERVED_NAMES if name == value.name reserved = true;
            for name: RESERVED_TYPE_NAMES if name == value.name reserved = true;

            if !reserved {
                if !looks_like_valid_identifier(value.name) || array_find(names, value.name) return false;
                array_add(*names, value.name);
                append_indentation(builder);
                print_to_builder(builder, "% :: %;\n", value.name, value.output_name);
            }
        }
    }

    for 0..generator.tab_depth - 2 append(builder, TAB_STRING);

    append(builder, "}");

    return !builder.failed;
}

fail_type_emission :: (builder: *String_Builder) {
    log_error("Bindings_Generator: unsupported or malformed type emission model\n");
    if builder builder.failed = true;
}

commit_type_fragment :: (destination: *String_Builder, fragment: *String_Builder, valid: bool) {
    if !destination || !valid || fragment.failed {
        fail_type_emission(destination);
        return;
    }

    text := builder_to_string(fragment);

    defer free(text);
    append(destination, text);
}

cpp_type :: (builder: *String_Builder, type: *CType, scope: *Declaration, depth := 0, omit_top_cv := false) -> bool {
    return cpp_declarator(builder, type, scope, "", 0, depth, omit_top_cv);
}

cpp_declarator :: (builder: *String_Builder, type: *CType, scope: *Declaration, name: string, flags: Print_Decl_Flags, depth: int, omit_cv: bool, public_shape := false, source_identity: *CType = null) -> bool {
    if !type || type.hardcoded_jai_string return false;

    identity := ifx source_identity then source_identity else type;
    if !BE_Traversal.enter(.CPP_TYPE, identity) return false;

    defer BE_Traversal.leave();

    if type.pointer_to {
        if public_shape && type.flags & .INSERTED return cpp_declarator(builder, type.pointer_to, scope, name, flags, depth + 1, false, true);

        modifier := ifx type.flags & .REFERENCE && !(public_shape && flags & .IN_FUNCTION_RETURN) then "&" else "*";

        if !omit_cv && !(type.flags & .REFERENCE) {
            if type.flags & .CONST modifier = tprint("% const ", modifier);
            if type.flags & .VOLATILE modifier = tprint("% volatile ", modifier);
        }

        declarator := tprint("%%", modifier, name);
        if (!public_shape && type.pointer_to.array_element_type) || type.pointer_to.type_of_function declarator = tprint("(%)", declarator);

        return cpp_declarator(builder, type.pointer_to, scope, declarator, flags, depth + 1, false, public_shape);
    }

    if type.flags & .REFERENCE return false;

    if type.array_element_type {
        if public_shape {
            declarator := tprint("*%", name);
            if type.array_element_type.type_of_function declarator = tprint("(%)", declarator);

            element := type.array_element_type.*;
            if !omit_cv element.flags |= type.flags & (CType.Flags.CONST | .VOLATILE);

            return cpp_declarator(builder, *element, scope, declarator, flags, depth + 1, false, true, type.array_element_type);
        }

        if type.array_element_count < -1 return false;
        if type.array_element_count < 0 && type.array_element_count_expression return false;

        declarator := ifx type.array_element_count < 0 then tprint("%[]", name) else tprint("%[%]", name, type.array_element_count);
        element := type.array_element_type.*;

        if !omit_cv element.flags |= type.flags & (CType.Flags.CONST | .VOLATILE);

        return cpp_declarator(builder, *element, scope, declarator, flags, depth + 1, false, false, type.array_element_type);
    }

    if type.type_of_function {
        function := type.type_of_function;
        if function.flags & (.CPP_METHOD | .CONSTRUCTOR | .DESTRUCTOR | .OBJC_METHOD | .OBJC_CLASS_METHOD) return false;
        if !function.return_type || function.return_type.array_element_type || function.return_type.type_of_function return false;

        parameters: String_Builder;

        defer free_buffers(*parameters);

        for argument, index: function.arguments {
            if !argument return false;
            if index > 0 append(*parameters, ", ");
            if !cpp_declarator(*parameters, argument.type, scope, "", (flags & ~Print_Decl_Flags.IN_FUNCTION_RETURN) | .IN_FUNCTION_ARGS, depth + 1, false, public_shape) return false;
        }

        if function.flags & .VARIADIC {
            if function.arguments.count append(*parameters, ", ");
            append(*parameters, "...");
        }

        text := builder_to_string(*parameters);

        defer free(text);
        declarator := tprint("%(%)", name, text);

        return cpp_declarator(builder, function.return_type, scope, declarator, (flags & ~Print_Decl_Flags.IN_FUNCTION_ARGS) | .IN_FUNCTION_RETURN, depth + 1, false, public_shape);
    }

    if flags & .NO_PATH && (type.type_of_struct || type.type_of_enum || type.type_of_typedef) {
        declaration: *Declaration;
        if type.type_of_struct declaration = cast(*Declaration) type.type_of_struct;
        if type.type_of_enum declaration = cast(*Declaration) type.type_of_enum;
        if type.type_of_typedef declaration = cast(*Declaration) type.type_of_typedef;
        if !cpp_identifier(declaration.name) return false;

        if !omit_cv {
            if type.flags & .CONST append(builder, "const ");
            if type.flags & .VOLATILE append(builder, "volatile ");
        }

        append(builder, declaration.name);
    } else if !cpp_scalar_type(builder, type, scope, omit_cv) return false;

    if name {
        append(builder, " ");
        append(builder, name);
    }

    return !builder.failed;
}

jai_named_type :: (builder: *String_Builder, declaration: *Declaration, scope: *Declaration, flags: Print_Decl_Flags) -> bool {
    if !declaration || !looks_like_valid_identifier(declaration.output_name) return false;

    parents: [..] *Declaration;

    defer array_free(parents);
    parent := declaration.parent;
    while parent && parent != scope && !(flags & .NO_PATH) {
        if array_find(parents, parent) return false;
        array_add(*parents, parent);
        parent = parent.parent;
    }

    for index: 0..parents.count - 1 {
        member := parents[parents.count - 1 - index];
        if !member.output_name continue;
        if member.kind == .NAMESPACE && (cast(*Namespace) member).flags & .FLATTEN continue;
        if member.kind != .STRUCT && member.kind != .NAMESPACE continue;
        if !looks_like_valid_identifier(member.output_name) return false;

        append(builder, member.output_name);
        append(builder, ".");
    }

    append(builder, declaration.output_name);

    return true;
}

binding_wide_integer_type :: (type: *CType) -> bool {
    return type != null && type.size == 16 && !type.pointer_to && !type.array_element_type && !type.type_of_struct && !type.type_of_enum && !type.type_of_typedef && !type.type_of_function && !type.type_of_declaration &&
           !(type.number_flags & .FLOAT) && !(type.flags & (.VOID | .BOOL)) &&
           (!type.hardcoded_jai_string || type.hardcoded_jai_string == "S128" || type.hardcoded_jai_string == "U128");
}

jai_model_type :: (builder: *String_Builder, type: *CType, scope: *Declaration, flags: Print_Decl_Flags, depth: int, existing: *[..] *Declaration = null) -> bool {
    if !type return false;
    if !BE_Traversal.enter(.JAI_TYPE, type) return false;

    defer BE_Traversal.leave();

    if type.hardcoded_jai_string {
        append(builder, type.hardcoded_jai_string);
        return true;
    }

    if type.type_of_struct || type.type_of_enum || type.type_of_typedef || type.type_of_declaration {
        declaration := query_named_declaration(type);

        if type.type_of_struct && (!declaration.output_name || type.type_of_struct.flags & .IS_ANON_INLINE_TYPE) return binding_struct_body(builder, type.type_of_struct, existing, 0);
        if type.type_of_enum && !declaration.output_name return binding_enum_body(builder, type.type_of_enum);

        if flags & .NO_PATH {
            name := get_type_name(type);
            if !looks_like_valid_identifier(name) return false;

            append(builder, name);

            if type.template_instantiation_params.count {
                formals: [] *Declaration;
                if type.type_of_struct formals = type.type_of_struct.template_type_params;
                return binding_template_arguments(builder, type.template_instantiation_params, formals, scope, depth + 1);
            }

            return true;
        }

        if !jai_named_type(builder, declaration, scope, flags) return false;

        formals: [] *Declaration;
        if type.type_of_struct formals = type.type_of_struct.template_type_params;

        return binding_template_arguments(builder, type.template_instantiation_params, formals, scope, depth + 1);
    }

    if type.pointer_to {
        if !type.pointer_to.type_of_function append(builder, "*");
        return jai_model_type(builder, type.pointer_to, scope, flags, depth + 1, existing);
    }

    if type.array_element_type {
        if type.array_element_count_expression {
            append(builder, "[");
            enum_printed := false;
            if !jai_expression_fragment(builder, type.array_element_count_expression, scope, false, *enum_printed, depth + 1) return false;
            append(builder, "] ");
        } else if type.array_element_count < 0 append(builder, "*");
        else print_to_builder(builder, "[%] ", type.array_element_count);

        return jai_model_type(builder, type.array_element_type, scope, flags, depth + 1, existing);
    }

    if type.type_of_function {
        if !jai_model_function(builder, type.type_of_function, scope, flags, depth + 1) return false;
        if !binding_valist_is_opaque(type.type_of_function) append_calling_convention_directives(builder, type.type_of_function, false);
        return true;
    }

    if type.flags & .VOID append(builder, "void");
    else if type.flags & .BOOL append(builder, "bool");
    else {
        bits := type.size * 8;

        if binding_wide_integer_type(type) {
            append(builder, ifx type.number_flags & .SIGNED then "S128" else "U128");
            return !builder.failed;
        }

        if !type.number_flags || (bits != 8 && bits != 16 && bits != 32 && bits != 64) return false;
        if type.number_flags & .FLOAT && bits != 32 && bits != 64 return false;

        prefix := ifx type.number_flags & .FLOAT then "float" else ifx type.number_flags & .SIGNED then "s" else "u";

        print_to_builder(builder, "%%", prefix, bits);
    }

    return !builder.failed;
}

jai_model_function :: (builder: *String_Builder, function: *Function_Type, scope: *Declaration, flags: Print_Decl_Flags, depth: int) -> bool {
    if !function return false;
    if !BE_Traversal.enter(.FUNCTION_TYPE, function) return false;

    defer BE_Traversal.leave();

    if binding_valist_is_opaque(function) {
        append(builder, "*void /* function containing va_list omitted by strip policy */");
        return true;
    }

    if function.arguments.count < 0 || function.arguments.count > 0x7fffffffffffffff / size_of(*Declaration) || (function.arguments.count > 0 && !function.arguments.data) return false;

    if !binding_value_type_supported(function.return_type) {
        log_error("Bindings_Generator: incomplete native record cannot be returned by value\n");
        return false;
    }

    for argument: function.arguments {
        if !argument || !binding_value_type_supported(argument.type) {
            log_error("Bindings_Generator: incomplete or malformed native argument cannot be emitted by value\n");
            return false;
        }
    }

    append(builder, "(");

    for argument, index: function.arguments {
        if !argument return false;
        if index > 0 append(builder, ", ");

        default_value := argument.expression != null && !(flags & .NO_ARGUMENT_DEFAULTS);
        normalized := argument.*;

        if default_value maybe_convert_expression_zero_to_null(*normalized);

        defer if normalized.expression != argument.expression free(normalized.expression);
        underlying := strip_type(argument.type, typedefs = true);
        omit_type := default_value && underlying && (underlying.flags & .BOOL) != 0;

        if argument.output_name {
            if !looks_like_valid_identifier(argument.output_name) return false;
            print_to_builder(builder, ifx omit_type then "% := " else "%: ", argument.output_name);
        }

        if omit_type && !argument.output_name return false;
        if !omit_type && !jai_model_type(builder, argument.type, scope, flags, depth + 1) return false;

        if default_value {
            if !argument.output_name return false;
            if !omit_type append(builder, " = ");

            enum_printed := false;

            if normalized.expression.kind == .LITERAL {
                if !jai_literal_fragment(builder, cast(*Literal) normalized.expression, argument.type, scope, false, *enum_printed) return false;
            } else if !jai_expression_fragment(builder, normalized.expression, scope, false, *enum_printed, depth + 1) return false;
        }
    }

    if function.flags & .VARIADIC {
        if !append_jai_variadic_tail(builder, function.arguments) return false;
    }

    append(builder, ") -> ");

    return jai_model_type(builder, function.return_type, scope, flags, depth + 1);
}

binding_valist_is_opaque :: (function: *Function_Type) -> bool {
    options := gp_current_options();
    if !function || !options || !(options.strip_flags & .FUNCTIONS_WITH_VALIST) return false;
    for argument: function.arguments if argument && is_va_list(argument.type) return true;
    return false;
}

append_jai_variadic_tail :: (builder: *String_Builder, arguments: [] *Declaration) -> bool {
    for suffix: 0..arguments.count {
        name := tprint("varargs_%", suffix);
        clash := false;

        for argument: arguments if argument && argument.output_name == name clash = true;
        if clash continue;
        if arguments.count > 0 append(builder, ", ");

        print_to_builder(builder, "%: ..Any", name);

        return !builder.failed;
    }

    return false;
}
