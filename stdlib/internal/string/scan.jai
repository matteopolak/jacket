scan :: (format: string, text: string) -> bool, [] Any {
    push_allocator(temp);
    values: [..] Any;
    rest := text;
    cursor: s64;

    while cursor < format.count {
        byte := format[cursor];

        cursor += 1;

        if byte != 37 {
            if rest.count == 0 || rest[0] != byte return false, values;
            advance(*rest);
            continue;
        }

        if cursor == format.count return false, values;

        directive := format[cursor];

        cursor += 1;

        if directive == 37 {
            if rest.count == 0 || rest[0] != 37 return false, values;
            // The scan contract intentionally leaves this byte unconsumed.
            continue;
        }

        value, ok := scan_value(directive, *rest);
        if !ok return false, values;

        array_add(*values, value);
    }

    return rest.count == 0, values;
}

scan2 :: (text: string, format: string, args: ..Any) -> bool {
    rest := text;
    cursor: s64;
    argument: s64;

    while cursor < format.count {
        byte := format[cursor];

        cursor += 1;

        if byte != 37 {
            if rest.count == 0 || rest[0] != byte return false;
            advance(*rest);
            continue;
        }

        if cursor == format.count return false;

        directive := format[cursor];

        cursor += 1;

        if directive == 37 {
            if rest.count == 0 || rest[0] != 37 return false;
            advance(*rest);
            continue;
        }

        if argument >= args.count return false;

        destination := args[argument];
        if !destination.type || !destination.value_pointer || destination.type.type != .POINTER return false;

        pointee := (cast(*Type_Info_Pointer) destination.type).pointer_to;
        wide_float := pointee.type == .FLOAT && pointee.runtime_size == 8;
        value, ok := scan_value(directive, *rest, wide_float);

        if !ok || !store_scan_value(args[argument], value) return false;

        argument += 1;
    }

    return rest.count == 0;
}

#scope_file
temporary_value :: (value: $T) -> Any {
    pointer := cast(*T) talloc(size_of(T));
    assert(pointer != null);
    pointer.* = value;
    return .{ type_info(T), pointer };
}

scan_value :: (directive: u8, text: *string, wide_float := false) -> Any, bool {
    if directive == 100 || directive == 105 {
        value, ok, rest := string_to_int(text.*);
        if !ok return .{}, false;
        text.* = rest;
        return temporary_value(value), true;
    }

    if directive == 102 {
        if wide_float {
            value, ok, rest := string_to_float64(text.*);
            if !ok return .{}, false;
            text.* = rest;
            return temporary_value(value), true;
        }

        value, ok, rest := string_to_float(text.*);
        if !ok return .{}, false;

        text.* = rest;

        return temporary_value(value), true;
    }

    if directive == 98 {
        value, ok := parse_bool(text);
        if !ok return .{}, false;
        return temporary_value(value), true;
    }

    if directive == 115 {
        if text.count == 0 || text.data[0] != 34 return .{}, false;

        end := find_index_from_left(text.*, cast(u8) 34, 1);
        if end < 0 return .{}, false;

        value := slice(text.*, 1, end - 1);

        advance(text, end + 1);

        return temporary_value(value), true;
    }

    return .{}, false;
}

store_scan_value :: (destination: Any, value: Any) -> bool {
    if !destination.type || !destination.value_pointer || destination.type.type != .POINTER return false;

    info := (cast(*Type_Info_Pointer) destination.type).pointer_to;
    pointer := (cast(**void) destination.value_pointer).*;

    if !pointer return false;

    if info == value.type {
        memcpy(pointer, value.value_pointer, info.runtime_size);
        return true;
    }

    if info.type == .FLOAT && value.type.type == .FLOAT {
        real := (cast(*float) value.value_pointer).*;

        if info.runtime_size == 8 {
            (cast(*float64) pointer).* = cast(float64) real;
            return true;
        }
    }

    if info.type == .INTEGER && value.type.type == .INTEGER {
        integer := cast(*Type_Info_Integer) info;
        number := (cast(*s64) value.value_pointer).*;

        if integer.signed {
            if info.runtime_size == 1 {
                if number < -128 || number > 127 return false;
                (cast(*s8) pointer).* = cast(s8) number;
                return true;
            }

            if info.runtime_size == 2 {
                if number < -32768 || number > 32767 return false;
                (cast(*s16) pointer).* = cast(s16) number;
                return true;
            }

            if info.runtime_size == 4 {
                if number < -2147483648 || number > 2147483647 return false;
                (cast(*s32) pointer).* = cast(s32) number;
                return true;
            }
        } else {
            if number < 0 return false;

            if info.runtime_size == 1 {
                if number > 255 return false;
                (cast(*u8) pointer).* = cast(u8) number;
                return true;
            }

            if info.runtime_size == 2 {
                if number > 65535 return false;
                (cast(*u16) pointer).* = cast(u16) number;
                return true;
            }

            if info.runtime_size == 4 {
                if number > 4294967295 return false;
                (cast(*u32) pointer).* = cast(u32) number;
                return true;
            }

            if info.runtime_size == 8 {
                (cast(*u64) pointer).* = cast(u64) number;
                return true;
            }
        }
    }

    return false;
}
