#import "Basic";
#import "String";

Pair :: struct {
    a: int;
    b: int;
}

Payload :: union {
    i: s64;
    f: float64;
}

Holder :: struct {
    tag: int;
    value: Payload;
}

Base :: struct {
    x: int;
    y: int;
}

Derived :: struct {
    z: int;
    using base: Base;
}

add :: (a: int, b: int) -> int {
    return a + b;
}

main :: () -> s64 {
    // type_of(null) is *void.
    if tprint("%", type_of(null)) != "*void" return 1;

    // Pointers print as bare hex with an underscore every four digits.
    p := cast(*void) 0x1234_5678_9abc;
    if tprint("%", p) != "1234_5678_9abc" return 2;

    q := cast(*void) 0xabc;
    if tprint("%", q) != "abc" return 3;
    if tprint("%", cast(*void) null) != "null" return 4;

    // The reviewed contract does not promise a concrete procedure-address
    // prefix. Preserve callable behavior and the specified null procedure form.
    text := tprint("%", add);
    if text.count == 0 || add(2, 3) != 5 return 5;

    missing: type_of(add);
    if tprint("%", missing) != "null" return 11;

    // The fallback spelling for a union is not established. Check its storage
    // independently and require the enclosing struct's known tag to be printed.
    h := Holder.{ 3, .{ i = 7 } };
    if h.value.i != 7 || !contains(tprint("%", h), "3") return 6;

    // Drawing a type name preserves that name and field values; no whitespace
    // between the name and delimiter is specified by the reviewed behavior.
    s := tprint("%", FormatStruct.{ value = Pair.{ 1, 2 }, draw_type_name = true });
    if !contains(s, "Pair") || !contains(s, "1") || !contains(s, "2") return 7;

    // get_field returns the member and its offset, searching `using` members.
    info := cast(*Type_Info_Struct) type_info(Derived);
    member, offset := get_field(info, "y");

    if !member || offset != 16 return 8;

    member, offset = get_field(info, "z");
    if !member || offset != 0 return 9;

    member, offset = get_field(info, "nothing");
    if member || offset != 0 return 10;

    return 0;
}
