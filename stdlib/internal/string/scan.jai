// Reads values from `text` as `format` describes. Returns whether the whole text matched, and the values
// read, as `Any`s.
//
// Bytes of `format` must appear in the text as they are. `%d` and `%i` read a whole number (an `s64`),
// `%f` a `float`, `%b` `true` or `false`, and `%s` a string in double quotes, returned without the
// quotes. A `%%` matches and consumes a literal `%` and produces no value.
//
// On a mismatch the result is `false` and the values read so far are returned. The values and the
// array are allocated in temporary storage, so they are only valid until it is reset. Nothing is
// freed by the caller.
//
// ```jai
// ok, values := scan("%d %s", "-8 \"yes\"");
// assert(ok && values.count == 2);
// assert((cast(*s64) values[0].value_pointer).* == -8);
// assert((cast(*string) values[1].value_pointer).* == "yes");
//
// percent_ok, percent_values := scan("%d%%", "50%");
// assert(percent_ok && percent_values.count == 1);
// ```
scan :: (format: string, text: string) -> bool, [] Any {
    push_allocator(temp);
    values: [..] Any;
    rest := text;
    cursor: s64;

    while cursor < format.count {
        byte := format[cursor];

        cursor += 1;

        if byte != #char "%" {
            if rest.count == 0 || rest[0] != byte return false, values;
            advance(*rest);
            continue;
        }

        if cursor == format.count return false, values;

        directive := format[cursor];

        cursor += 1;

        if directive == #char "%" {
            if rest.count == 0 || rest[0] != #char "%" return false, values;
            advance(*rest);
            continue;
        }

        value, ok := scan_value(directive, *rest);
        if !ok return false, values;

        array_add(*values, value);
    }

    return rest.count == 0, values;
}

// Reads values from `text` as `format` describes, storing them through the pointers in `args`.
// Returns whether the whole text matched.
//
// The directives are those of `scan`. Each
// directive stores into the next argument, which must be a pointer to a number, a `bool` or a string.
// A number that does not fit the target (a `256` into a `u8`) makes the call fail, as does a missing
// or null argument. On failure the targets read before the mismatch keep their new values and the
// others are not touched. A string result points into `text`.
//
// ```jai
// count: int;
// flag: bool;
// name: string;
// ok := scan2("7 true \"bob\"", "%d %b %s", *count, *flag, *name);
// assert(ok && count == 7 && flag && name == "bob");
// ```
//
// Floats read into a `float` or a `float64`, and a literal percent sign is written `%%`:
//
// ```jai
// ratio: float;
// precise: float64;
// assert(scan2("1.25%", "%f%%", *ratio) && ratio == 1.25);
// assert(scan2("1.23456789012345", "%f", *precise) && precise == 1.23456789012345);
// ```
scan2 :: (text: string, format: string, args: ..Any) -> bool {
    rest := text;
    cursor: s64;
    argument: s64;

    while cursor < format.count {
        byte := format[cursor];

        cursor += 1;

        if byte != #char "%" {
            if rest.count == 0 || rest[0] != byte return false;
            advance(*rest);
            continue;
        }

        if cursor == format.count return false;

        directive := format[cursor];

        cursor += 1;

        if directive == #char "%" {
            if rest.count == 0 || rest[0] != #char "%" return false;
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
    if directive == 100 || directive == #char "i" {
        value, ok, rest := string_to_int(text.*);
        if !ok return .{}, false;
        text.* = rest;
        return temporary_value(value), true;
    }

    if directive == #char "f" {
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

    if directive == #char "b" {
        value, ok := parse_bool(text);
        if !ok return .{}, false;
        return temporary_value(value), true;
    }

    if directive == #char "s" {
        if text.count == 0 || text.data[0] != #char "\"" return .{}, false;

        end := find_index_from_left(text.*, cast(u8) #char "\"", 1);
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
