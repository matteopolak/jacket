get_array_count_and_data :: (pointer: *void, info: *Type_Info_Array) -> (count: s64, data: *void) {
    if info.array_type == .FIXED return info.array_count, pointer;
    view := cast(*Array_View_64) pointer;
    return view.count, view.data;
}

signed_integer_range_from_size :: (size_in_bytes: int) -> (low: s64, high: s64) #no_aoc {
    assert(size_in_bytes == 1 || size_in_bytes == 2 || size_in_bytes == 4 || size_in_bytes == 8);
    high := cast(s64) (cast(u64) 0x7fffffffffffffff >> (64 - size_in_bytes * 8));
    return -high - 1, high;
}

unsigned_integer_range_from_size :: (size_in_bytes: int) -> (low: u64, high: u64) #no_aoc {
    assert(size_in_bytes == 1 || size_in_bytes == 2 || size_in_bytes == 4 || size_in_bytes == 8);
    return 0, 0xffffffffffffffff >> (64 - size_in_bytes * 8);
}

range_check_and_store :: (value: $T, info: *Type_Info_Integer, pointer: *void) -> (success: bool, low: T, high: T) {
    #if T == u64 {
        lo, hi := unsigned_integer_range_from_size(info.runtime_size);
        if info.signed hi >>= 1;
        if value > hi return false, lo, hi;

        store_integer(pointer, info.runtime_size, value);

        return true, lo, hi;
    } else {
        if info.signed {
            lo, hi := signed_integer_range_from_size(info.runtime_size);
            if value < lo || value > hi return false, cast(T) lo, cast(T) hi;
            store_integer(pointer, info.runtime_size, cast(u64) value);
            return true, cast(T) lo, cast(T) hi;
        }

        lo, hi := unsigned_integer_range_from_size(info.runtime_size);
        if value < 0 || cast(u64) value > hi return false, cast(T) lo, cast, truncate(T) hi;

        store_integer(pointer, info.runtime_size, cast(u64) value);

        return true, cast(T) lo, cast, truncate(T) hi;
    }
}

get_struct_field_info :: (info: *Type_Info_Struct, name: string) -> *Type_Info_Struct_Member {
    for *member: info.members if member.name == name return member;
    return null;
}

enum_name_to_value :: ($T: Type, name: string) -> T, bool {
    info := cast(*Type_Info_Enum) type_info(T);
    for n, i: info.names if n == name return cast(T) info.values[i], true;
    return cast(T) 0, false;
}

enum_value_to_name :: (e_value: $Enum) -> string {
    info := cast(*Type_Info_Enum) type_info(Enum);
    value := cast(s64) e_value;
    for v, i: info.values if v == value return info.names[i];
    return "";
}

get_enum_value :: (data: *void, info: *Type_Info_Enum) -> s64 {
    if info.internal_type.signed return get_s64_from_void_pointer(data, info.internal_type.runtime_size);
    return cast(s64) get_u64_from_void_pointer(data, info.internal_type.runtime_size);
}

set_enum_value :: (data: *void, info: *Type_Info_Enum, value: s64) {
    store_integer(data, info.internal_type.runtime_size, cast(u64) value);
}

set_value_from_string :: (any: Any, text: string) -> bool {
    cursor := text;
    return set_value_from_string(any, *cursor);
}

set_value_from_string :: (any: Any, text: *string) -> bool {
    info := any.type;
    pointer := any.value_pointer;

    if info.type == .BOOL {
        if text.count >= 4 && S.begins_with(text.*, "true") {
            (cast(*bool) pointer).* = true;
            advance(text, 4);
            return true;
        }

        if text.count >= 5 && S.begins_with(text.*, "false") {
            (cast(*bool) pointer).* = false;
            advance(text, 5);
            return true;
        }

        return false;
    }

    if info.type == .INTEGER {
        integer := cast(*Type_Info_Integer) info;
        if integer.signed {
            value, ok, rest := string_to_int(text.*, 10, s64);
            if !ok return false;

            text.* = rest;
            stored := range_check_and_store(value, integer, pointer);

            return stored;
        }

        value, ok, rest := string_to_int(text.*, 10, u64);
        if !ok return false;

        text.* = rest;
        stored := range_check_and_store(value, integer, pointer);

        return stored;
    }

    if info.type == .FLOAT {
        value, ok, rest := string_to_float64(text.*);
        if !ok return false;

        if info.runtime_size == 4 {
            (cast(*float32) pointer).* = cast(float32) value;
        } else if info.runtime_size == 8 {
            (cast(*float64) pointer).* = value;
        } else return false;

        text.* = rest;

        return true;
    }

    if info.type == .STRING {
        input := eat_spaces(text.*);
        if input.count == 0 || input[0] != 34 return false;

        end := S.find_index_from_left(input, cast(u8) 34, 1);
        if end < 0 return false;

        (cast(*string) pointer).* = copy_string(S.slice(input, 1, end - 1));
        text.* = S.slice(input, end + 1, input.count);

        return true;
    }

    if info.type == .ENUM {
        token, ok := S.parse_token(text);
        if !ok return false;

        enumeration := cast(*Type_Info_Enum) info;
        for name, i: enumeration.names if name == token {
            set_enum_value(pointer, enumeration, enumeration.values[i]);
            return true;
        }
    }

    return false;
}

#scope_module
#import "Basic";
S :: #import "String";

#scope_file
store_integer :: (pointer: *void, size: s64, value: u64) {
    if size == 1 {
        (cast(*u8) pointer).* = cast,truncate(u8) value;
    } else if size == 2 {
        (cast(*u16) pointer).* = cast,truncate(u16) value;
    } else if size == 4 {
        (cast(*u32) pointer).* = cast,truncate(u32) value;
    } else {
        assert(size == 8);
        (cast(*u64) pointer).* = value;
    }
}
