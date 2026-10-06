// Checks the rules of docs/language/unions.md.
// rules: union.1 union.2 union.3 union.4 union.5 union.6 union.7 union.8 union.9 union.10 union.11 union.12
#import "Basic";

Bits :: union {
    i: s32;
    f: float;
    b: [4] u8;
}

Mixed :: union {
    small: u8;
    big: [3] u16;
    wide: s64;
}

Shape :: enum u8 {
    NONE;
    CIRCLE;
    RECT;
}

Item :: union shape: Shape {
    .CIRCLE,, radius: float64;
    .RECT,, size: struct {
        w: s32;
        h: s32;
    };
}

Defaulted :: union shape: Shape = .RECT {
    .CIRCLE,, radius: float64;
    .RECT,, size: struct {
        w: s32;
        h: s32;
    };
}

Inferred :: union shape := Shape.CIRCLE {
    .CIRCLE,, radius: float64;
    .RECT,, size: struct {
        w: s32;
        h: s32;
    };
}

main :: () {
    // union.1: all members at offset 0.
    for type_info(Mixed).members assert(it.offset_in_bytes == 0, "union.1");

    // union.2, union.3: size of the largest member, alignment of the most aligned one.
    assert(size_of(Mixed) == 8, "union.2");
    Wrap :: struct {
        c: u8;
        m: Mixed;
    }
    assert(type_info(Wrap).members[1].offset_in_bytes == 8, "union.3");
    Wrap2 :: struct {
        c: u8;
        m: Bits;
    }
    assert(type_info(Wrap2).members[1].offset_in_bytes == 4, "union.3 four");

    // union.4: writing one member is visible through the others.
    uu: Bits;
    uu.f = 1.0;
    assert(tprint("% %", uu.i, size_of(Bits)) == "1065353216 4", "union.4");
    assert(uu.b[3] == 0x3f, "union.4 bytes");

    // union.5: the tag comes first, then the variants share one offset.
    info := type_info(Item);
    assert(info.type == .STRUCT, "union.5 struct");
    assert(info.members[0].name == "shape" && info.members[0].offset_in_bytes == 0, "union.5 tag");
    assert(info.members[1].offset_in_bytes == 8 && info.members[2].offset_in_bytes == 8, "union.5 variants");
    assert(size_of(Item) == 16, "union.5 size");

    // union.6, union.7: variants are reachable directly.
    a: Item = .{ shape = .RECT, size = .{ 3, 4 } };
    assert(a.shape == .RECT && a.size.w == 3 && a.size.h == 4, "union.6");
    a.shape = .CIRCLE;
    a.radius = 2.5;
    assert(a.radius == 2.5, "union.7");

    // union.8, union.9: a tag default, with an explicit or inferred tag type.
    d: Defaulted;
    assert(d.shape == .RECT, "union.9 explicit type");
    n: Inferred;
    assert(n.shape == .CIRCLE, "union.9 inferred type");
    assert(type_info(Inferred).members[0].type == type_info(Shape), "union.8 tag type");
    assert(size_of(Inferred) == 16, "union.8 size");

    // union.10: tagged-union flags.
    flags := info.textual_flags;
    assert((flags & .UNION) != 0 && (flags & .UNION_IS_TAGGED) != 0, "union.10");

    // union.11: members are the tag, then each variant.
    assert(info.members.count == 3 && info.members[1].name == "radius" && info.members[2].name == "size", "union.11");

    // union.12: bindings map tag values to member indices.
    assert(info.tagged_union_bindings.count == 2, "union.12 count");
    for info.tagged_union_bindings {
        if it.constant_value == xx Shape.CIRCLE assert(it.member_index == 1, "union.12 circle");
        else if it.constant_value == xx Shape.RECT assert(it.member_index == 2, "union.12 rect");
        else assert(false, "union.12 unexpected tag");
    }

    print("ok\n");
}
