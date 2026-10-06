// Checks the rules of docs/language/structs.md.
// rules: struct.1 struct.2 struct.3 struct.4 struct.6 struct.7 struct.8 struct.9 struct.10 struct.11 struct.12 struct.13 struct.14 struct.15 struct.16
#import "Basic";

Point3 :: struct {
    x: float = 1;
    y: float = 2;
    z: float = 3;
}

Gaps :: struct {
    a: u8;
    b: s64;
    c: u16;
}

Tight :: struct #no_padding {
    a: u8;
    b: s64;
}

Wide :: struct #align 16 {
    a: u8;
    b: s32;
}

Overlay :: struct {
    a: s32;
    #place a;
    b: s32;
    c: s32;
}

Color :: struct {
    union {
        using parts: struct {
            r, g, b, a: u8;
        };
        bytes: [4] u8;
        word: u32;
    }
}

Holder :: struct(T: Type, N: int) {
    items: [N] T;
}

WithConst :: struct {
    LIMIT :: 7;
    n: int;
}

Mode :: enum {
    A;
    B;
    C;
}
Base :: struct {
    mode: Mode = .A;
    extra: int = 11;
}
Child :: struct {
    using base: Base;
    mode = .C;
}

main :: () {
    // struct.1: a declared variable takes the member defaults.
    p: Point3;
    assert(p.x == 1 && p.y == 2 && p.z == 3, "struct.1");

    // struct.2: omitted literal members keep their defaults.
    w := Point3.{ y = 9 };
    assert(w.x == 1 && w.y == 9 && w.z == 3, "struct.2");

    // struct.3: positional literals follow declaration order.
    u := Point3.{ 4, 5, 6 };
    assert(u.x == 4 && u.y == 5 && u.z == 6, "struct.3");

    // struct.4: an empty literal is all defaults.
    e := Point3.{};
    assert(e.x == 1 && e.y == 2 && e.z == 3, "struct.4");

    // struct.6: natural alignment, size rounded to the struct alignment.
    assert(size_of(Gaps) == 24, "struct.6 size");
    info := type_info(Gaps);
    assert(info.members[1].offset_in_bytes == 8 && info.members[2].offset_in_bytes == 16, "struct.6 offsets");

    // struct.7: #no_padding packs the members.
    assert(size_of(Tight) == 9, "struct.7");
    assert(type_info(Tight).members[1].offset_in_bytes == 1, "struct.7 offset");

    // struct.8: #align raises the alignment and so the rounded size.
    assert(size_of(Wide) == 16, "struct.8");

    // struct.9: #place overlays b on a; c follows normally.
    o: Overlay;
    o.a = 1234;
    assert(o.b == 1234, "struct.9 overlay");
    assert(size_of(Overlay) == 8, "struct.9 size");

    // struct.10, struct.11, struct.12: anonymous union members are reachable directly,
    // using-struct members through the name and directly, all sharing storage.
    c: Color;
    c.word = 0x04030201;
    assert(c.r == 1 && c.a == 4, "struct.10");
    assert(c.parts.g == 2 && c.g == 2, "struct.11");
    assert(c.bytes[3] == 4 && size_of(Color) == 4, "struct.12");

    // struct.13: instances are cached per argument value.
    assert(Holder(int, 2) == Holder(s64, 2), "struct.13 same");
    assert(Holder(int, 2) != Holder(int, 3), "struct.13 differ");

    // struct.14: instance names show parameter names.
    assert(tprint("%", Holder(int, 3)) == "Holder(T=s64, N=3)", "struct.14");

    // struct.15: constants in a struct body.
    assert(WithConst.LIMIT == 7 && size_of(WithConst) == 8, "struct.15");

    // struct.16: a member-override statement in a struct body.
    ch: Child;
    assert(ch.mode == .C && ch.extra == 11, "struct.16");

    print("ok\n");
}
