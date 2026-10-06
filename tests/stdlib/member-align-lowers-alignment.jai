// A member's `#align N` sets its alignment, also below the type's natural one (like C's
// `#pragma pack`). Windows.jai relies on it: `FILETIME` is 4-aligned, so `WIN32_FIND_DATAW`
// puts `cFileName` at byte 44 as the C header does.
#import "Basic";

Packed :: struct {
    a: u32;
    b: u64 #align 4;
}
Filetime_Like :: struct {
    lo: u32;
    hi: u32;
    #place lo;
    quad: u64 = --- #align 4;
}
Holder :: struct {
    tag: u32;
    time: Filetime_Like;
    next: u32;
}
Raised :: struct {
    a: u8;
    b: u32 #align 16;
}

main :: () {
    assert(size_of(Packed) == 12);
    assert(type_info(Packed).members[1].offset_in_bytes == 4);
    assert(size_of(Filetime_Like) == 8);
    assert(type_info(Holder).members[1].offset_in_bytes == 4);
    assert(size_of(Holder) == 16);
    assert(type_info(Raised).members[1].offset_in_bytes == 16);
    assert(size_of(Raised) == 32);
    print("ok\n");
}
