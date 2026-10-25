// After deinit the array is empty (nothing iterates, no freed storage is reachable) and can be
// set up again with init_bit_array.
#import "Basic";
#import "Extensions/Testing";

deinit_leaves_an_empty_array :: () {
    bits := Bits.make_bit_array(70);

    Bits.set_all_bits(*bits);
    Bits.deinit(*bits);
    expect_equal(bits.count, 0);
    expect(bits.slots.count == 0 && bits.slots.data == null);

    visited := 0;

    for bits visited += 1;

    expect_equal(visited, 0);
} @test

an_array_can_be_set_up_again_after_deinit :: () {
    bits := Bits.make_bit_array(70);

    Bits.deinit(*bits);
    Bits.init_bit_array(*bits, 10);
    Bits.set_bit(*bits, 9);
    expect(bits.count == 10 && bits.slots.count == 1 && bits.slots[0] == 1 << 9);
    Bits.deinit(*bits);
} @test

#scope_file

Bits :: #import "Bit_Array";
