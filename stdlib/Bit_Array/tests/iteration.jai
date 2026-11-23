// Plain, set-only and cleared-only iteration over a 130-bit array whose set bits cross word boundaries.
#import "Basic";
Bits :: #import "Bit_Array";

// The bits that are set in the test array: 0, 63, 64 and 129.
is_set_index :: (i: int) -> bool {
    return i == 0 || i == 63 || i == 64 || i == 129;
}

main :: () {
    bits := Bits.make_bit_array(130);

    defer Bits.deinit(*bits);

    positions := int.[0, 63, 64, 129];
    for positions Bits.set_bit(*bits, it);

    for bits assert(it == is_set_index(it_index));

    ones: [..] int;

    defer array_free(ones);
    for: Bits.only_set bits array_add(*ones, it_index);

    assert(ones.count == 4);
    for i: 0..3 assert(ones[i] == positions[i]);

    unset := 0;

    for: Bits.only_unset bits {
        assert(!it);
        assert(!is_set_index(it_index));
        unset += 1;
    }

    assert(unset == 126);

    Bits.set_all_bits(*bits);
    unset = 0;
    for: Bits.only_unset bits unset += 1;

    assert(unset == 0);

    set := 0;
    for bits if it set += 1;

    assert(set == 130);

    Bits.clear_all_bits(*bits);
    set = 0;
    for: Bits.only_set bits set += 1;

    assert(set == 0);

    print("ok\n");
}
