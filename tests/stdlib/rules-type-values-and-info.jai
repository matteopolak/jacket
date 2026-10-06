// Checks the rules of docs/language/type-values-and-info.md.
// rules: typeval.1 typeval.2 typeval.3 typeval.4 typeval.5 typeval.6 typeval.7 typeval.9 typeval.10 typeval.11 typeval.12 typeval.13 typeval.14
#import "Basic";

Base :: struct {
    a: int;
}
Derived :: struct {
    using #as base: Base;
    b: float @tagged;
}
Same :: struct {
    a: int;
}
Twin :: struct {
    a: int;
}
Alias :: Base;
Count :: #type,distinct s64;
Ratio :: #type,isa float;
Box :: struct(T: Type) {
    v: T;
}
Fruit :: enum {
    APPLE;
    PEAR;
}
Noted :: struct @persist @v2 {
    x: int;
}

half :: (f: float) -> float {
    return f / 2;
}

main :: () {
    // typeval.1: structural types are interned.
    d: Derived;
    assert(*Derived == type_of(*d), "typeval.1 pointer");
    assert([4] int == [4] s64 && [] u8 != [] s8, "typeval.1 arrays");
    assert(type_of(half) == #type (float) -> float, "typeval.1 proc");

    // typeval.2: nominal types are distinct per declaration.
    assert(Same != Twin, "typeval.2 structs");

    // typeval.3: polymorphic struct instances are cached per argument.
    assert(Box(int) == Box(s64) && Box(int) != Box(u8), "typeval.3");

    // typeval.4: an alias is the same type.
    assert(Alias == Base, "typeval.4");
    assert(type_of(d) == Derived, "typeval.4 type_of");

    // typeval.5: distinct is a new type with the same size.
    assert(Count != s64 && size_of(Count) == 8, "typeval.5");

    // typeval.6: isa converts implicitly to its base.
    r: Ratio = 3;
    assert(half(r) == 1.5, "typeval.6");

    // typeval.7: Type values are first-class.
    tt: [2] Type = .[int, float];
    assert(tprint("% %", tt[0], tt[1]) == "s64 float32", "typeval.7 print");
    held := Fruit;
    assert(held == Fruit && held != tt[0], "typeval.7 compare");

    // typeval.9: sizes of Type and Any.
    assert(size_of(Type) == 8 && size_of(Any) == 16, "typeval.9");

    // typeval.10: Any.type is a *Type_Info.
    any: Any = 5;
    assert(any.type == type_info(int), "typeval.10");
    assert(any.type.type == .INTEGER, "typeval.10 tag");

    // typeval.11: the info tag.
    assert(type_info(Base).type == .STRUCT, "typeval.11 struct");
    assert(type_info([3] int).type == .ARRAY, "typeval.11 array");
    assert(type_info(u16).type == .INTEGER, "typeval.11 integer");
    assert(type_info(Fruit).type == .ENUM, "typeval.11 enum");

    // typeval.12: the struct name.
    assert(type_info(Derived).name == "Derived", "typeval.12");

    // typeval.13: members with name, offset, type and flags.
    m := type_info(Derived).members;
    assert(m.count == 2 && m[0].name == "base" && m[1].name == "b", "typeval.13 names");
    assert(m[0].offset_in_bytes == 0 && m[1].offset_in_bytes == 8, "typeval.13 offsets");
    assert(m[1].type == type_info(float), "typeval.13 type");
    assert((m[0].flags & .USING) != 0 && (m[0].flags & .AS) != 0, "typeval.13 flags");
    assert((m[1].flags & .USING) == 0, "typeval.13 plain flags");

    // typeval.14: member notes and struct notes.
    assert(m[1].notes.count == 1 && m[1].notes[0] == "tagged", "typeval.14 member");
    sn := type_info(Noted).notes;
    assert(sn.count == 2 && sn[0] == "persist" && sn[1] == "v2", "typeval.14 struct");

    print("ok\n");
}
