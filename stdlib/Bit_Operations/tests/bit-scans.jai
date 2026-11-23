// bit_scan_forward and bit_scan_reverse return 1 + the bit index, and 0 for zero, at every width.
#import "Basic";
B :: #import "Bit_Operations";

main :: () {
    for k: 0..63 {
        bit := (cast(u64) 1) << cast(u64) k;
        assert(B.bit_scan_forward(bit) == cast(s32) (k + 1));
        assert(B.bit_scan_reverse(bit) == cast(s32) (k + 1));
    }

    assert(B.bit_scan_forward(cast(u8) 0b1010_0000) == 6);
    assert(B.bit_scan_reverse(cast(u8) 0b1010_0000) == 8);
    assert(B.bit_scan_forward(cast(u16) 0x8000) == 16);
    assert(B.bit_scan_reverse(cast(u16) 0x0001) == 1);
    assert(B.bit_scan_reverse(cast(u32) 0b1000) == 4);

    // Negative signed values are scanned as their two's-complement bits of the type's width.
    assert(B.bit_scan_forward(cast(s8) -128) == 8);
    assert(B.bit_scan_forward(cast(s64) -1) == 1);
    s64_min := -9223372036854775807 - 1;

    assert(B.bit_scan_forward(s64_min) == 64);
    assert(B.bit_scan_reverse(cast(s8) -1) == 8);
    assert(B.bit_scan_reverse(cast(s16) -1) == 16);
    assert(B.bit_scan_reverse(cast(s64) -1) == 64);
    assert(B.bit_scan_reverse(cast(u64) 0xFFFF_FFFF_FFFF_FFFF) == 64);

    // Zero has no set bit at any width.
    assert(B.bit_scan_forward(cast(u8) 0) == 0);
    assert(B.bit_scan_forward(cast(u16) 0) == 0);
    assert(B.bit_scan_forward(cast(u32) 0) == 0);
    assert(B.bit_scan_forward(cast(u64) 0) == 0);
    assert(B.bit_scan_forward(cast(s8) 0) == 0);
    assert(B.bit_scan_forward(cast(s64) 0) == 0);
    assert(B.bit_scan_reverse(cast(u8) 0) == 0);
    assert(B.bit_scan_reverse(cast(u16) 0) == 0);
    assert(B.bit_scan_reverse(cast(u32) 0) == 0);
    assert(B.bit_scan_reverse(cast(u64) 0) == 0);
    assert(B.bit_scan_reverse(cast(s8) 0) == 0);
    assert(B.bit_scan_reverse(cast(s64) 0) == 0);

    print("ok\n");
}
