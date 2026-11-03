// Whether `T` is a number, pointer or enum, the types `min`, `max` and `clamp` accept.
//
// ```jai
// assert(IsScalar(int) && IsScalar(*u8));
// assert(!IsScalar(string));
// ```
IsScalar :: (T: Type) -> bool #compile_time {
    info := cast(*Type_Info) T;
    return info.type == .INTEGER || info.type == .FLOAT || info.type == .POINTER || info.type == .ENUM;
}

// The lowest and highest value of the integer type `T`.
//
// ```jai
// low, high := get_integer_range(u8);
// assert(low == 0 && high == 255);
// low_s, high_s := get_integer_range(s8);
// assert(low_s == -128 && high_s == 127);
// ```
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

// Converts `value` to `TargetType`, using the nearest value that fits when it is out of range, where a plain cast would wrap.
//
// ```jai
// big := 1000;
// assert(clamp_to_another_integer_type(big, u8) == 255);
// assert(clamp_to_another_integer_type(-5, u8) == 0);
// assert(clamp_to_another_integer_type(100, u8) == 100);
// ```
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

// The names of the members of the enum `T`, in declaration order.
//
// ```jai
// Color :: enum {
//     RED;
//     GREEN;
//     BLUE;
// }
// names := enum_names(Color);
// assert(names.count == 3 && names[1] == "GREEN");
// ```
enum_names :: ($t: Type) -> [] string {
    return enum_info(t).names;
}

// The values of the members of the enum `T`, in declaration order, as `s64`.
//
// ```jai
// Level :: enum {
//     LOW :: 1;
//     MID :: 5;
//     HIGH :: 9;
// }
// values := enum_values_as_s64(Level);
// assert(values.count == 3 && values[2] == 9);
// ```
enum_values_as_s64 :: ($T: Type) -> [] s64 {
    return enum_info(T).values;
}

// The members of the enum `T`, in declaration order. The slice lives in temporary storage.
//
// ```jai
// Color :: enum {
//     RED;
//     GREEN;
//     BLUE;
// }
// members := enum_values_as_enum(Color);
// assert(members.count == 3 && members[2] == .BLUE);
// ```
enum_values_as_enum :: ($T: Type) -> [] T {
    info := enum_info(T);
    result := ([] T).{ info.values.count, cast(*T) talloc(info.values.count * size_of(T)) };

    assert(result.data != null || result.count == 0);
    for value, index: info.values result[index] = cast,no_check(T) value;

    return result;
}

// Finds the member called `name` in a struct, looking through `using` members too. Returns the member and its byte offset
// from the start of the struct, or null and 0 when there is none.
//
// ```jai
// Point :: struct {
//     x: int;
//     y: int;
// }
// info := cast(*Type_Info_Struct) type_info(Point);
// member, offset := get_field(info, "y");
// assert(member != null && offset == 8);
// ```
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

// Like `enum_range`, for an enum whose `Type_Info_Enum` you already hold. An enum with no members gives `0, 0`.
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

// The lowest and highest member value of the enum `t`.
//
// ```jai
// Level :: enum {
//     LOW :: 1;
//     MID :: 5;
//     HIGH :: 9;
// }
// low, high := enum_range(Level);
// assert(low == 1 && high == 9);
// ```
enum_range :: ($t: Type) -> (low: s64, high: s64) {
    return enum_range_given_info(enum_info(t));
}

// The highest member value of the enum `t`.
//
// ```jai
// Level :: enum {
//     LOW :: 1;
//     MID :: 5;
//     HIGH :: 9;
// }
// assert(enum_highest_value(Level) == 9);
// ```
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
