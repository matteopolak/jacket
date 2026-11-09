// Json_Value: a parsed document of unknown shape, and the procedures that build and read one.

// What a [Json_Value] holds.
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
//
// ```jai
// doc := json_object();
// json_set(*doc, "name", json_string("Ada"));
// tags := json_array();
// json_push(*tags, json_int(1));
// json_set(*doc, "tags", tags);
//
// text := to_json(doc);
// assert(text == "{\"name\":\"Ada\",\"tags\":[1]}");
// free(text);
// free_json(*doc);
// ```
Json_Value :: struct {
    kind: Json_Kind;
    boolean: bool;
    text: string;
    items: [..] Json_Value;
    members: [..] Json_Member;
}

// One `key: value` of an object.
Json_Member :: struct {
    key: string;
    value: Json_Value;
}

// A JSON `null`.
json_null :: () -> Json_Value {
    return .{};
}

// A JSON `true` or `false`.
json_bool :: (value: bool) -> Json_Value {
    result: Json_Value;
    result.kind = .BOOL;
    result.boolean = value;

    return result;
}

// A JSON number from a signed integer.
json_int :: (value: s64) -> Json_Value {
    result: Json_Value;
    result.kind = .NUMBER;
    result.text = sprint("%", value);

    return result;
}

// A JSON number from an unsigned integer.
json_uint :: (value: u64) -> Json_Value {
    result: Json_Value;
    result.kind = .NUMBER;
    result.text = sprint("%", value);

    return result;
}

// NaN and infinities have no JSON spelling and become null.
//
// ```jai
// assert(json_float(1.5).text == "1.5");
// zero := 0.0;
// assert(json_float(zero / zero).kind == .NULL);
// ```
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

// A JSON string. The string is copied.
//
// ```jai
// v := json_string("hi");
// text, ok := json_as_string(*v);
// assert(ok && text == "hi");
// free_json(*v);
// ```
json_string :: (value: string) -> Json_Value {
    result: Json_Value;
    result.kind = .STRING;
    result.text = copy_string(value);

    return result;
}

// An empty JSON array; fill it with [json_push].
json_array :: () -> Json_Value {
    result: Json_Value;
    result.kind = .ARRAY;

    return result;
}

// An empty JSON object; fill it with [json_set].
json_object :: () -> Json_Value {
    result: Json_Value;
    result.kind = .OBJECT;

    return result;
}

// Appends to an array; `array` takes ownership of `item`.
//
// ```jai
// list := json_array();
// json_push(*list, json_bool(true));
// assert(json_count(*list) == 1);
// free_json(*list);
// ```
json_push :: (array: *Json_Value, item: Json_Value) {
    assert(array.kind == .ARRAY, "json_push needs an array");

    array_add(*array.items, item);
}

// Sets a member of an object, replacing the last member with that key; `object` takes
// ownership of `value` and copies `key`.
//
// - key: the member name.
// - value: moved into `object`, which owns it from then on.
//
// ```jai
// object := json_object();
// json_set(*object, "n", json_int(1));
// json_set(*object, "n", json_int(2));
// assert(json_count(*object) == 1);
// free_json(*object);
// ```
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
//
// ```jai
// object, _ := from_json(Json_Value, "{\"n\": 5}");
// n := json_get(*object, "n");
// value, ok := json_as_s64(n);
// assert(ok && value == 5);
// assert(json_get(*object, "missing") == null);
// free_json(*object);
// ```
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
//
// ```jai
// list, _ := from_json(Json_Value, "[10, 20]");
// second, _ := json_as_s64(json_at(*list, 1));
// assert(second == 20);
// assert(json_at(*list, 2) == null);
// free_json(*list);
// ```
json_at :: (array: *Json_Value, index: int) -> *Json_Value {
    if array.kind != .ARRAY || index < 0 || index >= array.items.count return null;

    return *array.items[index];
}

// How many elements an array has, or members an object has; 0 for anything else.
//
// ```jai
// list, _ := from_json(Json_Value, "[1, 2, 3]");
// assert(json_count(*list) == 3);
// free_json(*list);
// ```
json_count :: (value: *Json_Value) -> int {
    if value.kind == .ARRAY return value.items.count;
    if value.kind == .OBJECT return value.members.count;

    return 0;
}

// A number that is written as an integer and fits. False for anything else.
//
// ```jai
// n, _ := from_json(Json_Value, "42");
// value, ok := json_as_s64(*n);
// assert(ok && value == 42);
//
// fraction, _ := from_json(Json_Value, "4.5");
// _, ok = json_as_s64(*fraction);
// assert(!ok);
// free_json(*n);
// free_json(*fraction);
// ```
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

// The same as [json_as_s64], for an unsigned integer.
json_as_u64 :: (value: *Json_Value) -> u64, bool {
    if value.kind != .NUMBER return 0, false;

    negative, magnitude, ok := parse_integer_token(value.text);

    if !ok || (negative && magnitude != 0) return 0, false;

    return magnitude, true;
}

// Any number, rounded to the nearest float64. False if it is not a number or is too large.
//
// ```jai
// n, _ := from_json(Json_Value, "4.5");
// value, ok := json_as_float64(*n);
// assert(ok && value == 4.5);
// free_json(*n);
// ```
json_as_float64 :: (value: *Json_Value) -> float64, bool {
    if value.kind != .NUMBER return 0, false;

    result, overflow := float64_from_token(value.text);

    if overflow return 0, false;

    return result, true;
}

// The boolean and true, or false and false when the value is not a boolean.
json_as_bool :: (value: *Json_Value) -> bool, bool {
    if value.kind != .BOOL return false, false;

    return value.boolean, true;
}

// The text and true, or "" and false when the value is not a string. The text belongs to the value.
json_as_string :: (value: *Json_Value) -> string, bool {
    if value.kind != .STRING return "", false;

    return value.text, true;
}

// Releases everything `value` owns and leaves it null.
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
