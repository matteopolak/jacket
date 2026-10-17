print_type_to_builder :: (builder: *String_Builder, info: *Type_Info, print_parameters_matching_defaults := false) {
    render_type(builder, info, print_parameters_matching_defaults, formatter_style());
}

type_to_string :: (info: *Type_Info) -> string {
    builder: String_Builder;

    defer free_buffers(*builder);
    print_type_to_builder(*builder, info);
    push_allocator(temp);

    return builder_to_string(*builder);
}

#scope_module
render_type :: (builder: *String_Builder, info: *Type_Info, print_defaults: bool, style: *Print_Style) {
    assert(info != null, "a null type descriptor has no type name");

    if info.type == .INTEGER {
        integer := cast(*Type_Info_Integer) info;
        append(builder, ifx integer.signed then "s" else "u");
        print_u64_base_n(builder, cast(u64) info.runtime_size * 8, FormatInt.{});
    } else if info.type == .FLOAT {
        append(builder, "float");
        print_u64_base_n(builder, cast(u64) info.runtime_size * 8, FormatInt.{});
    } else if info.type == .STRUCT {
        record := cast(*Type_Info_Struct) info;

        if record.name.count > 0 append(builder, record.name);
        else append(builder, "(anonymous struct)");

        // Explicit parameter descriptors preserve type/value identity, not source text.
        if record.specified_parameters.count > 0 {
            append(builder, cast(u8) 40);
            printed: s64;

            for parameter: record.specified_parameters {
                assert(parameter.offset_into_constant_storage >= 0);
                item := Any.{ parameter.type, record.constant_storage.data + parameter.offset_into_constant_storage };
                matches := false;

                if !print_defaults {
                    assert(record.parameters.count > 0, "compiler must reflect declared generic parameters and defaults");

                    for declared: record.parameters {
                        if declared.name != parameter.name || declared.offset_into_constant_storage < 0 continue;
                        default := Any.{ declared.type, record.constant_storage.data + declared.offset_into_constant_storage };
                        matches = parameter_equal(item, default);
                        break;
                    }
                }

                if matches continue;
                if printed > 0 append(builder, ", ");

                append(builder, parameter.name);
                append(builder, cast(u8) 61);
                render_item(builder, item, true, false, style);
                printed += 1;
            }

            append(builder, cast(u8) 41);
        }
    } else if info.type == .ENUM append(builder, (cast(*Type_Info_Enum) info).name);
    else if info.type == .VARIANT append(builder, (cast(*Type_Info_Variant) info).name);
    else if info.type == .POINTER {
        target := (cast(*Type_Info_Pointer) info).pointer_to;
        if !target {
            append(builder, "null");
            return;
        }

        append(builder, cast(u8) 42);
        render_type(builder, target, print_defaults, style);
    } else if info.type == .ARRAY {
        array := cast(*Type_Info_Array) info;

        append(builder, cast(u8) 91);
        if array.array_type == .RESIZABLE append(builder, "..");
        else if array.array_type == .FIXED print_u64_base_n(builder, cast(u64) array.array_count, FormatInt.{});

        append(builder, "] ");
        render_type(builder, array.element_type, print_defaults, style);
    } else if info.type == .PROCEDURE {
        procedure := cast(*Type_Info_Procedure) info;

        append(builder, "procedure ");
        append(builder, cast(u8) 40);

        for argument, index: procedure.argument_types {
            if index > 0 append(builder, ", ");
            render_type(builder, argument, print_defaults, style);
        }

        append(builder, cast(u8) 41);

        if procedure.return_types.count > 0 {
            append(builder, " -> ");

            for result, index: procedure.return_types {
                if index > 0 append(builder, ", ");
                render_type(builder, result, print_defaults, style);
            }
        }

        if (procedure.procedure_flags & .IS_C_CALL) != 0 append(builder, " #c_call");
        else if (procedure.procedure_flags & .HAS_NO_CONTEXT) != 0 append(builder, " #no_context");
    } else if info.type == .BOOL append(builder, "bool");
    else if info.type == .STRING append(builder, "string");
    else if info.type == .TYPE append(builder, "Type");
    else if info.type == .ANY append(builder, "Any");
    else if info.type == .VOID append(builder, "void");
    else if info.type == .CODE append(builder, "Code");
    else if info.type == .OVERLOAD_SET append(builder, "(procedure overloads)");
    else assert(false, "unsupported reflected type name");
}

#scope_file
parameter_equal :: (a: Any, b: Any) -> bool {
    if a.type != b.type return false;
    if !a.type || a.type.runtime_size == 0 return true;

    assert(a.value_pointer != null && b.value_pointer != null);
    kind := a.type.type;
    if kind == .STRING return (cast(*string) a.value_pointer).* == (cast(*string) b.value_pointer).*;

    if kind == .FLOAT {
        if a.type.runtime_size == 4 return (cast(*float32) a.value_pointer).* == (cast(*float32) b.value_pointer).*;
        return (cast(*float64) a.value_pointer).* == (cast(*float64) b.value_pointer).*;
    }

    if kind == .STRUCT {
        info := cast(*Type_Info_Struct) a.type;
        for member: info.members {
            if (member.flags & (.CONSTANT | .IMPORTED | .OVERLAY)) != 0 || member.type.runtime_size == 0 continue;
            left := Any.{ member.type, cast(*u8) a.value_pointer + member.offset_in_bytes };
            right := Any.{ member.type, cast(*u8) b.value_pointer + member.offset_in_bytes };
            if !parameter_equal(left, right) return false;
        }

        return true;
    }

    if kind == .VARIANT {
        underlying := (cast(*Type_Info_Variant) a.type).variant_of;
        return parameter_equal(Any.{ underlying, a.value_pointer }, Any.{ underlying, b.value_pointer });
    }

    if kind == .ARRAY {
        info := cast(*Type_Info_Array) a.type;
        count := info.array_count;
        left := cast(*u8) a.value_pointer;
        right := cast(*u8) b.value_pointer;

        if info.array_type != .FIXED {
            x := (cast(*Array_View_64) a.value_pointer).*;
            y := (cast(*Array_View_64) b.value_pointer).*;

            if x.count != y.count return false;

            count = x.count;
            left = x.data;
            right = y.data;
        }

        for i: 0..count - 1 if !parameter_equal(Any.{ info.element_type, left + i * info.element_type.runtime_size }, Any.{ info.element_type, right + i * info.element_type.runtime_size }) return false;

        return true;
    }

    if kind == .ANY return parameter_equal((cast(*Any) a.value_pointer).*, (cast(*Any) b.value_pointer).*);

    left := cast(*u8) a.value_pointer;
    right := cast(*u8) b.value_pointer;

    for i: 0..a.type.runtime_size - 1 if left[i] != right[i] return false;

    return true;
}
