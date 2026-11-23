// Zero-length arrays, init_bit_array on an empty value, and reuse after deinit.
#import "Basic";
Bits :: #import "Bit_Array";

main :: () {
    empty := Bits.make_bit_array(0);

    assert(empty.count == 0);
    visited := 0;
    for empty visited += 1;

    assert(visited == 0);
    Bits.set_all_bits(*empty);
    Bits.toggle_all_bits(*empty);
    Bits.deinit(*empty);
    assert(empty.count == 0);

    bits: Bits.Bit_Array;

    Bits.init_bit_array(*bits, 3);
    assert(bits.count == 3 && !bits[0] && !bits[2]);

    // Toggling flips the three logical bits; toggling again restores them.
    Bits.toggle_all_bits(*bits);
    assert(bits[0] && bits[1] && bits[2]);
    set := 0;
    for bits if it set += 1;

    assert(set == 3);
    Bits.toggle_all_bits(*bits);
    for bits assert(!it);

    Bits.deinit(*bits);

    // After deinit the same value can hold a new array of another length.
    Bits.init_bit_array(*bits, 70);
    assert(bits.count == 70 && bits.slots.count == 2);
    assert(!bits[69]);

    // Setting or clearing a bit twice gives the same result as once.
    Bits.set_bit_to(*bits, 69, true);
    Bits.set_bit_to(*bits, 69, true);
    assert(bits[69]);
    Bits.set_bit_to(*bits, 69, false);
    assert(!bits[69]);

    Bits.set_bit(*bits, 0);
    Bits.set_bit(*bits, 0);
    Bits.clear_bit(*bits, 0);
    Bits.clear_bit(*bits, 0);
    assert(!bits[0]);

    Bits.deinit(*bits);
    assert(bits.count == 0);

    print("ok\n");
}
