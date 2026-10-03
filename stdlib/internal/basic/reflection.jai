IsScalar :: (T: Type) -> bool #compile_time {
    info := cast(*Type_Info) T;
    return info.type == .INTEGER || info.type == .FLOAT || info.type == .POINTER || info.type == .ENUM;
}

get_integer_range :: ($T: Type) -> (low: T, high: T) {
    info := type_info(T);

    assert(info.type == .INTEGER, "get_integer_range requires an integer type");
    integer := cast(*Type_Info_Integer) info;

    assert(info.runtime_size > 0 && info.runtime_size <= 8);
    maximum: u64 = 0xffffffffffffffff;
    maximum = maximum >> (64 - info.runtime_size * 8);

    if integer.signed {
        maximum = maximum >> 1;
        return cast,no_check(T) (~maximum), cast,no_check(T) maximum;
    }

    return cast(T) 0, cast,no_check(T) maximum;
}

clamp_to_another_integer_type :: (value: $SourceType, $TargetType: Type) -> TargetType {
    source := type_info(SourceType);
    target := type_info(TargetType);

    assert(source.type == .INTEGER && target.type == .INTEGER, "integer clamping requires integer types");
    low, high := get_integer_range(TargetType);
    source_integer := cast(*Type_Info_Integer) source;
    target_integer := cast(*Type_Info_Integer) target;

    if source_integer.signed {
        signed := cast(s64) value;
        if signed < 0 {
            if !target_integer.signed return low;
            if signed < cast(s64) low return low;
            return cast,no_check(TargetType) signed;
        }
    }

    unsigned := cast(u64) value;
    if unsigned > cast(u64) high return high;

    return cast,no_check(TargetType) unsigned;
}

enum_names :: ($t: Type) -> [] string {
    return enum_info(t).names;
}

enum_values_as_s64 :: ($T: Type) -> [] s64 {
    return enum_info(T).values;
}

enum_values_as_enum :: ($T: Type) -> [] T {
    info := enum_info(T);
    result := ([] T).{ info.values.count, cast(*T) talloc(info.values.count * size_of(T)) };

    assert(result.data != null || result.count == 0);
    for value, index: info.values result[index] = cast,no_check(T) value;

    return result;
}

get_field :: (info: *Type_Info_Struct, name: string) -> *Type_Info_Struct_Member, offset_in_bytes: s64 {
    assert(info != null);
    for *member: info.members if member.name == name return member, member.offset_in_bytes;

    for member: info.members {
        if (member.flags & .USING) == 0 || member.offset_in_bytes < 0 || member.type.type != .STRUCT continue;
        found, nested_offset := get_field(cast(*Type_Info_Struct) member.type, name);
        if found return found, member.offset_in_bytes + nested_offset;
    }

    return null, 0;
}

enum_range_given_info :: (info: *Type_Info_Enum) -> (low: s64, high: s64) {
    assert(info != null);
    if info.values.count == 0 return 0, 0;

    low := info.values[0];
    high := low;

    for value: info.values {
        low = min(low, value);
        high = max(high, value);
    }

    return low, high;
}

enum_range :: ($t: Type) -> (low: s64, high: s64) {
    return enum_range_given_info(enum_info(t));
}

enum_highest_value :: ($t: Type) -> s64 {
    low, high := enum_range(t);
    return high;
}

#scope_module
enum_info :: ($T: Type) -> *Type_Info_Enum {
    info := type_info(T);
    assert(info.type == .ENUM, "enum reflection requires an enum type");
    return cast(*Type_Info_Enum) info;
}
