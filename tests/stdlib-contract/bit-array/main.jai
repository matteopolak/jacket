// Logical indexing and documented padding-bit behavior.
#import "Basic";
Bit_Array :: #import "Bit_Array";

main :: () {
    bits: Bit_Array.Bit_Array = Bit_Array.make_bit_array(65);

    assert(bits.count == 65 && bits.allocator.proc != null);
    Bit_Array.set_all_bits(*bits);
    assert(bits.slots.count == 2);
    assert(bits.slots[0] == -1 && bits.slots[1] == -1);
    Bit_Array.clear_bit(*bits, 64);
    assert(bits.slots[1] == -2);
    count := 0;

    for bits {
        assert(it == (it_index != 64));
        count += 1;
    }

    assert(count == 65);
    Bit_Array.deinit(*bits);
    print("ok\n");
}
