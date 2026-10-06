// `#overlay(f)` members share `f`'s storage and carry `Type_Info_Struct_Member.Flags.OVERLAY`
// (sjorsdonkers/toml-jai skips them when serializing).
// rules: struct.18
#import "Basic";

Pixel :: struct {
    r, g, b, a: u8;
    #overlay(r) rgba: u32;
    after: u16;
}

flags_of :: (info: *Type_Info_Struct, name: string) -> Type_Info_Struct_Member.Flags {
    for info.members if it.name == name return it.flags;
    assert(false, "no member %", name);
    return 0;
}

main :: () {
    p: Pixel;
    p.rgba = 0x04030201;
    assert(p.r == 1 && p.a == 4);
    assert(size_of(Pixel) == 8);

    info := type_info(Pixel);
    for info.members if it.name == "after" assert(it.offset_in_bytes == 4);
    assert((flags_of(info, "rgba") & .OVERLAY) != 0);
    assert((flags_of(info, "r") & .OVERLAY) == 0);
    assert((flags_of(info, "after") & .OVERLAY) == 0);

    printed := 0;
    for info.members {
        if it.flags & (.CONSTANT | .IMPORTED | .OVERLAY) continue;
        printed += 1;
    }
    assert(printed == 5);
    print("ok\n");
}
