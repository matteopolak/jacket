Argument_Flags :: enum_flags u64 {
    FREE_ARGUMENTS_ALLOWED :: 1;
    DOUBLE_DASH_REQUIRED :: 2;
    SHOW_HELP_ON_ERROR :: 4;
    SORT_HELP :: 8;
    ALIGN_HELP :: 16;
}

Default_Argument_Flags :: Argument_Flags.FREE_ARGUMENTS_ALLOWED | .SORT_HELP | .ALIGN_HELP;

Is_Set :: struct(T: Type) {
    #insert #run make_fields(T);
}

parse_arguments :: ($T: Type, flags := Default_Argument_Flags, help_triggers: [] string = .["help", "HELP", "?"], args: [] string = .{ count = -1 }) -> (success: bool, result: T, is_set: Is_Set(T), free_arguments: [..] string) {
    info := cast(*Type_Info_Struct) type_info(T);

    assert(info.type == .STRUCT);
    values: T;
    assigned: Is_Set(T);
    free_args: [..] string;
    arguments := args;
    acquired := args.count == -1;
    index: s64;

    if acquired {
        arguments = get_command_line_arguments();
        index = 1;
    }

    defer {
        if acquired array_free(arguments);
    }

    prefix := "-";
    if (flags & .DOUBLE_DASH_REQUIRED) != 0 prefix = "--";

    free_only := false;

    while index < arguments.count {
        arg := arguments[index];

        index += 1;

        if !free_only && arg == "--" && (flags & .FREE_ARGUMENTS_ALLOWED) != 0 {
            free_only = true;
            continue;
        }

        name := arg;
        option := !free_only && S.begins_with(arg, prefix);

        if option name = S.slice(arg, prefix.count, arg.count);

        if !free_only for trigger: help_triggers if name == trigger {
            show_help(T, flags, help_triggers);
            return false, values, assigned, free_args;
        }

        if !option {
            if (flags & .FREE_ARGUMENTS_ALLOWED) == 0 {
                show_if_requested(T, flags, help_triggers);
                return false, values, assigned, free_args;
            }

            array_add(*free_args, arg);
            continue;
        }

        member := R.get_struct_field_info(info, name);
        if !member || (member.flags & .CONSTANT) != 0 {
            show_if_requested(T, flags, help_triggers);
            return false, values, assigned, free_args;
        }

        set_member := R.get_struct_field_info(type_info(Is_Set(T)), name);

        assert(set_member != null);
        set_pointer := cast(*bool) ((cast(*u8) *assigned) + set_member.offset_in_bytes);
        if set_pointer.* {
            show_if_requested(T, flags, help_triggers);
            return false, values, assigned, free_args;
        }

        set_pointer.* = true;
        dest := (cast(*u8) *values) + member.offset_in_bytes;

        if member.type.type == .BOOL {
            (cast(*bool) dest).* = true;
            continue;
        }

        if member.type.type == .ARRAY {
            array_info := cast(*Type_Info_Array) member.type;
            if array_info.array_type != .FIXED {
                show_if_requested(T, flags, help_triggers);
                return false, values, assigned, free_args;
            }

            for element: 0..array_info.array_count - 1 {
                if index >= arguments.count || !store_argument(array_info.element_type, dest + element * array_info.element_type.runtime_size, arguments[index]) {
                    show_if_requested(T, flags, help_triggers);
                    return false, values, assigned, free_args;
                }

                index += 1;
            }
        } else {
            if index >= arguments.count || !store_argument(member.type, dest, arguments[index]) {
                show_if_requested(T, flags, help_triggers);
                return false, values, assigned, free_args;
            }

            index += 1;
        }
    }

    return true, values, assigned, free_args;
}

show_help :: ($T: Type, flags: Argument_Flags, help_triggers: [] string) {
    info := cast(*Type_Info_Struct) type_info(T);
    members := array_copy(info.members);

    defer array_free(members);
    if (flags & .SORT_HELP) != 0 Sort.quick_sort(members, (a, b) => S.compare(a.name, b.name));

    longest: s64;
    for member: members if (member.flags & .CONSTANT) == 0 longest = max(longest, member.name.count);

    prefix := "-";
    if (flags & .DOUBLE_DASH_REQUIRED) != 0 prefix = "--";

    for member: members {
        if (member.flags & .CONSTANT) != 0 continue;
        for note: member.notes if note.count > 0 && note[0] == 63 write_strings(S.slice(note, 1, note.count), "\n");

        write_strings("  ", prefix, member.name);
        if (flags & .ALIGN_HELP) != 0 for 0..longest - member.name.count write_string(" ");
        else write_string(" ");
        for note: member.notes if note.count == 0 || note[0] != 63 write_strings(note, " ");

        write_string("\n");
    }

    if help_triggers.count > 0 {
        write_string("  help:");
        for trigger: help_triggers write_strings(" ", prefix, trigger);
        write_string("\n");
    }
}

#scope_file
#import "Basic";
R :: #import "Reflection";
S :: #import "String";
Sort :: #import "Sort";

make_fields :: ($T: Type) -> string {
    info := cast(*Type_Info_Struct) type_info(T);

    assert(info.type == .STRUCT);
    builder: String_Builder;

    defer free_buffers(*builder);

    for member: info.members if (member.flags & .CONSTANT) == 0 {
        append(*builder, member.name);
        append(*builder, ": bool = false;\n");
    }

    return builder_to_string(*builder);
}

store_argument :: (info: *Type_Info, dest: *void, value: string) -> bool {
    if info.type == .STRING {
        (cast(*string) dest).* = value;
        return true;
    }

    if info.type == .ENUM && ((cast(*Type_Info_Enum) info).enum_type_flags & .FLAGS) != 0 return false;

    raw := Any_Struct.{ info, dest };
    any := (cast(*Any) *raw).*;
    rest := value;

    return R.set_value_from_string(any, *rest) && rest.count == 0;
}

show_if_requested :: ($T: Type, flags: Argument_Flags, triggers: [] string) {
    if (flags & .SHOW_HELP_ON_ERROR) != 0 show_help(T, flags, triggers);
}
