input_name_to_input_target :: (name: string) -> SDL_Input_Target {
    for candidate, index: input_names {
        if name == candidate return cast(SDL_Input_Target) (index + 1);
    }

    return .UNINITIALIZED;
}

deinit :: (map: SDL_Gamepad_Mapping) {
    free(map.guid);
    free(map.name);
    free(map.platform);
    array_free(map.mappings);
}

get_guid_string :: (bustype: s16, vendor: s16, product: s16, version: s16) -> temp_string: string {
    result := talloc_string(32);
    for i: 0..31 result[i] = #char "0";

    values := u16.[cast,truncate(u16) bustype, cast,truncate(u16) vendor, cast,truncate(u16) product, cast,truncate(u16) version];
    for value, i: values {
        offset := i * 8;
        result[offset] = hex_digits[(value >> 4) & 15];
        result[offset + 1] = hex_digits[value & 15];
        result[offset + 2] = hex_digits[(value >> 12) & 15];
        result[offset + 3] = hex_digits[(value >> 8) & 15];
    }

    return result;
}

parse_gamepad_mapping_string :: (input: string) -> map: SDL_Gamepad_Mapping, success: bool {
    result: SDL_Gamepad_Mapping;
    committed := false;

    defer if !committed deinit(result);
    cursor: int = 0;
    guid, has_name := take_field(input, *cursor);

    if !has_name || guid.count == 0 return .{}, false;

    name, has_mappings := take_field(input, *cursor);
    if name.count == 0 || !has_mappings return .{}, false;

    result.guid = copy_string(guid);
    result.name = copy_string(name);

    while cursor < input.count {
        field, more := take_field(input, *cursor);
        if field.count == 0 {
            if more return .{}, false;
            break;
        }

        colon: int = 0;
        while colon < field.count && field[colon] != #char ":" colon += 1;
        if colon == field.count || colon == 0 || colon + 1 == field.count return .{}, false;

        target_name := slice(field, 0, colon);
        source := slice(field, colon + 1, field.count);

        if target_name == "platform" {
            free(result.platform);
            result.platform = copy_string(source);
            continue;
        }

        entry: SDL_Input_Map;
        entry.target = input_name_to_input_target(target_name);
        if entry.target == .UNINITIALIZED return .{}, false;

        number_start: int = 1;

        if source[0] == #char "a" entry.type = .AXIS;
        else if source[0] == #char "b" entry.type = .BUTTON;
        else if source[0] == #char "h" entry.type = .HAT;
        else return .{}, false;

        if entry.type == .HAT {
            dot := number_start;
            while dot < source.count && source[dot] != #char "." dot += 1;
            if dot == source.count return .{}, false;

            ok, value := parse_index(slice(source, number_start, dot));
            if !ok return .{}, false;

            entry.index = value;
            ok, value = parse_index(slice(source, dot + 1, source.count));
            if !ok || value == 0 || value > 15 return .{}, false;

            entry.subindex = value;
        } else {
            ok, value := parse_index(slice(source, number_start, source.count));
            if !ok return .{}, false;
            entry.index = value;
        }

        array_add(*result.mappings, entry);
    }

    committed = true;

    return result, true;
}

#scope_module
input_names := string.[
    "leftx", "rightx", "lefty", "righty", "lefttrigger", "righttrigger",
    "dpleft", "dpright", "dpup", "dpdown", "back", "guide", "start",
    "leftshoulder", "rightshoulder", "leftstick", "rightstick", "a", "b", "x", "y"
];

hex_digits :: "0123456789abcdef";

slice :: (s: string, begin: int, end: int) -> string {
    return .{ end - begin, s.data + begin };
}

take_field :: (input: string, cursor: *int) -> string, bool {
    begin := cursor.*;

    while cursor.* < input.count && input[cursor.*] != #char "," cursor.* += 1;

    result := slice(input, begin, cursor.*);
    more := cursor.* < input.count;

    if more cursor.* += 1;

    return result, more;
}

parse_index :: (s: string) -> bool, s32 {
    if s.count == 0 return false, 0;

    result: s32 = 0;

    for c: s {
        if c < #char "0" || c > #char "9" return false, 0;
        digit := cast(s32) c - #char "0";
        if result > (0x7fffffff - digit) / 10 return false, 0;
        result = result * 10 + digit;
    }

    return true, result;
}
