#import "Basic";
#import "Math";
#import "String";

Entity :: struct (Payload: Type) {
    value: Payload;
}

Box :: struct (T: Type) {
    item: T;
}

Bits :: union {
    i: s64;
    f: float64;
}

add :: (a: s64, b: s64) -> s64 {
    return a + b;
}

nothing :: () {}

main :: () -> s64 {
    // Procedure types carry a `procedure` prefix; values add a hex address.
    if tprint("%", type_of(add)) != "procedure (s64, s64) -> s64" return 1;
    if tprint("%", type_of(nothing)) != "procedure ()" return 2;

    text := tprint("%", add);
    if !begins_with(text, "procedure 0x") return 3;

    // Polymorphic struct arguments are spelled without spaces around `=`.
    if tprint("%", Box(s64)) != "Box(T=s64)" return 4;
    if tprint("%", Entity(struct { name := "x"; })) != "Entity(Payload=(anonymous struct))" return 5;

    // An untagged union does not know which member is live; a zeroed Type is void.
    bits: Bits;
    if tprint("%", bits) != "(untagged union)" return 6;

    zero: Type;
    if tprint("%", zero) != "void" return 7;

    // Large floats keep 18 significant digits and then zeros.
    big: float64 = 5.97219e24;
    if tprint("%", cast(float32) big) != "5972189887017193070000000" return 8;
    wide: float64 = 1.0e22;
    if tprint("%", wide) != "10000000000000000000000" return 9;
    if tprint("%", 123456789.0) != "123456792" return 10;

    // Members are listed in declaration order, overlays included.
    names: [..] string;
    for member: (cast(*Type_Info_Struct) type_info(Vector3)).members array_add(*names, member.name);

    if names.count != 6 || names[3] != "xy" || names[4] != "yz" || names[5] != "component" return 11;

    // Temporary storage rounds allocations to 8 bytes and grows its newest allocation in place.
    reset_temporary_storage();
    ts := context.temporary_storage;

    a := cast(*u8) talloc(20);
    if ts.total_bytes_occupied != 24 return 12;

    a = cast(*u8) temporary_allocator_proc(.RESIZE, 40, 20, a, null);
    if ts.total_bytes_occupied != 40 return 13;

    reset_temporary_storage();
    return 0;
}
