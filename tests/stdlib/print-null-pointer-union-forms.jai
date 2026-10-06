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

    // Procedure values print as "procedure 0x" followed by the address.
    text := tprint("%", add);
    if begins_with(text, "procedure 0x") == false return 5;

    // A union member prints as "(union)".
    h := Holder.{ 3, .{ i = 7 } };
    if tprint("%", h) != "{3, (union)}" return 6;

    // The type name follows the begin string without a space.
    s := tprint("%", FormatStruct.{ value = Pair.{ 1, 2 }, draw_type_name = true });
    if s != "Pair{1, 2}" return 7;

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
