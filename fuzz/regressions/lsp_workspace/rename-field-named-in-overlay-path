// rules: struct.24
// `#overlay(a.b)` and `#place a.b` anchor on a member reached through a field, here one inside
// an embedded struct that is not at the start of the struct.
#import "Basic";

Header :: struct {
    tag: u32;
    table: *u8;
}

Object :: struct {
    pad: u64;
    using base: Header;
    #overlay(base.table) typed: *u32;
    after: u16;
}

Placed :: struct {
    pad: u64;
    base: Header;
    #place base.table;
    other: *u32;
}

main :: () {
    o: Object;

    assert(offset_of(Object, "typed") == offset_of(Object, "table"));
    assert(offset_of(Object, "typed") == 16);
    assert(offset_of(Object, "after") == 24);

    value: u32 = 7;

    o.typed = *value;
    assert(o.table == cast(*u8) *value);

    assert(offset_of(Placed, "other") == 16);
}
