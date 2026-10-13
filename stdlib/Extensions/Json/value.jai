// Json_Value: a parsed document of unknown shape, and the procedures that build and read one.

Json_Kind :: enum u8 {
    NULL;
    BOOL;
    NUMBER;
    STRING;
    ARRAY;
    OBJECT;
}

// One JSON value. Only the fields of its kind are used:
//   BOOL    boolean
//   NUMBER  text, the number as written (exact for integers of any size; see json_as_s64)
//   STRING  text, the contents with escapes resolved
//   ARRAY   items
//   OBJECT  members, in document order (duplicate keys are kept; json_get finds the last)
// Everything a Json_Value owns comes from context.allocator; free_json releases it.
Json_Value :: struct {
    kind: Json_Kind;
    boolean: bool;
    text: string;
    items: [..] Json_Value;
    members: [..] Json_Member;
}

Json_Member :: struct {
    key: string;
    value: Json_Value;
}

json_null :: () -> Json_Value {
    return .{};
}

json_bool :: (value: bool) -> Json_Value {
    result: Json_Value;
    result.kind = .BOOL;
    result.boolean = value;

    return result;
}

json_int :: (value: s64) -> Json_Value {
    result: Json_Value;
    result.kind = .NUMBER;
    result.text = sprint("%", value);

    return result;
}

json_uint :: (value: u64) -> Json_Value {
    result: Json_Value;
    result.kind = .NUMBER;
    result.text = sprint("%", value);

    return result;
}

// NaN and infinities have no JSON spelling and become null.
json_float :: (value: float64) -> Json_Value {
    result: Json_Value;
    result.kind = .NUMBER;

    builder: String_Builder;

    append_float64(*builder, value);
    result.text = builder_to_string(*builder);

    if result.text == "null" {
        free(result.text);
        result.text = "";
        result.kind = .NULL;
    }

    return result;
}

// The string is copied.
json_string :: (value: string) -> Json_Value {
    result: Json_Value;
    result.kind = .STRING;
    result.text = copy_string(value);

    return result;
}

json_array :: () -> Json_Value {
    result: Json_Value;
    result.kind = .ARRAY;

    return result;
}

json_object :: () -> Json_Value {
    result: Json_Value;
    result.kind = .OBJECT;

    return result;
}

// Appends to an array; `array` takes ownership of `item`.
json_push :: (array: *Json_Value, item: Json_Value) {
    assert(array.kind == .ARRAY, "json_push needs an array");

    array_add(*array.items, item);
}

// Sets a member of an object, replacing the last member with that key; `object` takes
// ownership of `value` and copies `key`.
json_set :: (object: *Json_Value, key: string, value: Json_Value) {
    assert(object.kind == .OBJECT, "json_set needs an object");

    existing := json_get(object, key);

    if existing {
        free_json(existing);
        existing.* = value;

        return;
    }

    member: Json_Member;
    member.key = copy_string(key);
    member.value = value;
    array_add(*object.members, member);
}

// The member `key` of an object (the last one, if the key repeats), or null.
json_get :: (object: *Json_Value, key: string) -> *Json_Value {
    if object.kind != .OBJECT return null;

    i := object.members.count - 1;

    while i >= 0 {
        if object.members[i].key == key return *object.members[i].value;

        i -= 1;
    }

    return null;
}

// Element `index` of an array, or null.
json_at :: (array: *Json_Value, index: int) -> *Json_Value {
    if array.kind != .ARRAY || index < 0 || index >= array.items.count return null;

    return *array.items[index];
}

// How many elements an array has, or members an object has; 0 for anything else.
json_count :: (value: *Json_Value) -> int {
    if value.kind == .ARRAY return value.items.count;
    if value.kind == .OBJECT return value.members.count;

    return 0;
}

// A number that is written as an integer and fits. False for anything else.
json_as_s64 :: (value: *Json_Value) -> s64, bool {
    if value.kind != .NUMBER return 0, false;

    negative, magnitude, ok := parse_integer_token(value.text);

    if !ok return 0, false;

    if negative {
        if magnitude > 0x8000_0000_0000_0000 return 0, false;

        return cast,no_check(s64) (~magnitude + 1), true;
    }

    if magnitude > 0x7fff_ffff_ffff_ffff return 0, false;

    return cast,no_check(s64) magnitude, true;
}

json_as_u64 :: (value: *Json_Value) -> u64, bool {
    if value.kind != .NUMBER return 0, false;

    negative, magnitude, ok := parse_integer_token(value.text);

    if !ok || (negative && magnitude != 0) return 0, false;

    return magnitude, true;
}

// Any number, rounded to the nearest float64. False if it is not a number or is too large.
json_as_float64 :: (value: *Json_Value) -> float64, bool {
    if value.kind != .NUMBER return 0, false;

    result, overflow := float64_from_token(value.text);

    if overflow return 0, false;

    return result, true;
}

json_as_bool :: (value: *Json_Value) -> bool, bool {
    if value.kind != .BOOL return false, false;

    return value.boolean, true;
}

json_as_string :: (value: *Json_Value) -> string, bool {
    if value.kind != .STRING return "", false;

    return value.text, true;
}

// Releases everything `value` owns and leaves it null. Part of the `free_json` set.
free_json :: (value: *Json_Value) {
    if value.kind == .STRING || value.kind == .NUMBER {
        if value.text.count free(value.text);
    }

    for *item: value.items free_json(item);

    array_free(value.items);

    for *member: value.members {
        if member.key.count free(member.key);

        free_json(*member.value);
    }

    array_free(value.members);

    value.* = .{};
}
