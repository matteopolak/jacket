#import "Basic";
B :: #import "BuildCpp";

X64 :: 0x8664;
ARM64 :: 0xaa64;

ordinary :: (machine: u16) -> [32] u8 {
    header: [32] u8;
    header[0] = cast(u8) (machine & 0xff);
    header[1] = cast(u8) (machine >> 8);
    header[2] = 3; // section count

    return header;
}

bigobj :: (machine: u16) -> [32] u8 {
    header: [32] u8;
    header[2] = 0xff;
    header[3] = 0xff;
    header[4] = 2; // version
    header[6] = cast(u8) (machine & 0xff);
    header[7] = cast(u8) (machine >> 8);

    class := u8.[0xc7, 0xa1, 0xba, 0xd1, 0xee, 0xba, 0xa9, 0x4b, 0xaf, 0x20, 0xfa, 0xf6, 0x6a, 0xa4, 0xdc, 0xb8];
    for class header[12 + it_index] = it;

    return header;
}

prefix :: (header: *[32] u8, count: s64) -> [] u8 {
    view: [] u8;
    view.data = header.data;
    view.count = count;
    return view;
}

#run {
    header := ordinary(X64);

    assert(B.coff_object_matches(header, X64));
    assert(!B.coff_object_matches(header, ARM64));
    header = ordinary(ARM64);
    assert(B.coff_object_matches(header, ARM64));
    assert(!B.coff_object_matches(header, X64));
    assert(!B.coff_object_matches(prefix(*header, 19), ARM64));

    header = bigobj(X64);
    assert(B.coff_object_matches(header, X64));
    assert(!B.coff_object_matches(header, ARM64));
    header = bigobj(ARM64);
    assert(B.coff_object_matches(header, ARM64));
    assert(!B.coff_object_matches(header, X64));

    // A wrong class id is not a bigobj, and the unknown machine 0 is not an ordinary object.
    header[20] ^= 0xff;
    assert(!B.coff_object_matches(header, ARM64));
    assert(!B.coff_object_matches(prefix(*header, 31), ARM64));

    print("ok\n");
}
