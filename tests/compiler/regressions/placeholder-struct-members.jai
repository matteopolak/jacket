// `_` is a placeholder member name: a struct can have several, each taking space but none
// reachable by name.
#import "Basic";

Padded :: struct {
    _: u32;
    value: s64;
    _: u32;
    _: u8;
}

main :: () {
    p: Padded;
    p.value = 5;
    assert(p.value == 5);
    assert(size_of(Padded) == 24);

    info := cast(*Type_Info_Struct) type_info(Padded);

    assert(info.members.count == 4);
    assert(info.members[2].offset_in_bytes == 16);
    assert(info.members[3].offset_in_bytes == 20);

    print("ok\n");
}
