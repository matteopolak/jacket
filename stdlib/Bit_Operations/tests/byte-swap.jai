// byte_swap and byte_swap_in_place reverse the bytes of integers of each width and of a struct.
#import "Basic";
B :: #import "Bit_Operations";

Pair :: struct {
    a: u16;
    b: u16;
}

main :: () {
    assert(B.byte_swap(cast(u8) 0xAB) == 0xAB);
    assert(B.byte_swap(cast(u16) 0x0102) == 0x0201);
    assert(B.byte_swap(cast(u16) 0x00ff) == 0xff00);
    assert(B.byte_swap(cast(u32) 0x11223344) == 0x44332211);
    assert(B.byte_swap(cast(u64) 0x0102030405060708) == 0x0807060504030201);
    assert(B.byte_swap(cast(s16) -2) == -257);

    // The whole value is reversed, so the bytes of `a` move into `b`'s place and the other way round.
    p := Pair.{ 0x1122, 0x3344 };
    q := B.byte_swap(p);

    assert(q.a == 0x4433 && q.b == 0x2211);

    input := cast(u32) 0x11223344;
    swapped := B.byte_swap(input);

    assert(input == 0x11223344);
    assert(swapped == 0x44332211);

    mix: u64 = 0x9E37_79B9_7F4A_7C15;

    for i: 0..999 {
        x := (cast(u64) i) * mix;
        assert(B.byte_swap(B.byte_swap(x)) == x);
    }

    value: u32 = 0xAABBCCDD;

    B.byte_swap_in_place(*value);
    assert(value == 0xDDCCBBAA);
    B.byte_swap_in_place(*value);
    assert(value == 0xAABBCCDD);

    print("ok\n");
}
