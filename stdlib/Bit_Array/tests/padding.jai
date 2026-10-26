// Logical indexing and documented padding-bit behavior.
#import "Basic";
#import "Extensions/Testing";

set_all_bits_fills_every_slot :: () {
    bits: Bits.Bit_Array = Bits.make_bit_array(65);

    defer Bits.deinit(*bits);

    assert_that(bits.count, is(65));
    assert_that(bits.allocator.proc, not_null());
    Bits.set_all_bits(*bits);
    assert_that(bits.slots, has_length(2));
    expect(bits.slots[0] == -1 && bits.slots[1] == -1);
} @test

clearing_a_bit_and_iterating_use_logical_indices :: () {
    bits: Bits.Bit_Array = Bits.make_bit_array(65);

    defer Bits.deinit(*bits);

    Bits.set_all_bits(*bits);
    Bits.clear_bit(*bits, 64);
    expect_equal(bits.slots[1], -2);
    count := 0;

    for bits {
        expect(it == (it_index != 64));
        count += 1;
    }

    expect_equal(count, 65);
} @test

#scope_file

Bits :: #import "Bit_Array";
