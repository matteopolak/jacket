// `offset_of` gives a member's byte offset as a constant; `is_value_type` says whether a type's
// bytes hold the whole value.
// rules: struct.20 struct.21
#import "Basic";

Inner :: struct { x: u8; y: s32; }
Outer :: struct {
    tag: u8;
    big: s64;
    small: u16;
    inner: Inner;
    using base: Inner;
    packed: [3] u8;
}
Un :: union { a: u8; b: s64; }
Packed :: struct #no_padding { a: u8; b: s32; }

Strs :: struct { n: int; s: string; }
Ptrs :: struct { p: *int; }
Dyn :: struct { arr: [..] int; }
View :: struct { v: [] u8; }
Arr :: struct { a: [4] Strs; }
Nested :: struct { i: Inner; f: [2] float; }
En :: enum u8 { A; B; }
Dist :: #type,distinct s32;

main :: () {
    #run assert(offset_of(Outer, "tag") == 0);
    #run assert(offset_of(Outer, "big") == 8);
    #run assert(offset_of(Outer, "small") == 16);
    #run assert(offset_of(Outer, "inner") == 20);
    #run assert(offset_of(Outer, "inner.y") == 24);
    #run assert(offset_of(Outer, "y") == offset_of(Outer, "base") + 4);
    assert(offset_of(Outer, "big") == 8);
    assert(offset_of(Outer.small) == 16);
    assert(offset_of(Un, "b") == 0);
    assert(offset_of(Packed, "b") == 1);
    assert(type_of(offset_of(Outer, "big")) == s64);

    o: Outer;
    assert(cast(*u8) *o.big - cast(*u8) *o == offset_of(Outer, "big"));
    assert(cast(*u8) *o.y - cast(*u8) *o == offset_of(Outer, "y"));
    info := type_info(Outer);
    for info.members if it.name == "small" assert(it.offset_in_bytes == offset_of(Outer, "small"));

    // Use as a constant: an array length.
    Buf :: [offset_of(Outer, "big")] u8;
    assert(size_of(Buf) == 8);

    assert(is_value_type(int));
    assert(is_value_type(float64));
    assert(is_value_type(bool));
    assert(is_value_type(En));
    assert(is_value_type(Dist));
    assert(is_value_type(Inner));
    assert(is_value_type(Un));
    assert(is_value_type(Nested));
    assert(is_value_type([4] Inner));
    assert(!is_value_type(string));
    assert(!is_value_type(*int));
    assert(!is_value_type(Strs));
    assert(!is_value_type(Ptrs));
    assert(!is_value_type(Dyn));
    assert(!is_value_type(View));
    assert(!is_value_type(Arr));
    assert(!is_value_type([] int));
    assert(!is_value_type([..] int));
    assert(!is_value_type(Any));
    assert(!is_value_type(Type));
    assert(!is_value_type(() -> ()));
    #run assert(is_value_type(Inner) && !is_value_type(Strs));
    print("ok\n");
}
