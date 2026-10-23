// After deinit the array is empty (nothing iterates, no freed storage is reachable) and can be
// set up again with init_bit_array.
#import "Basic";
Bit_Array :: #import "Bit_Array";

main :: () {
    bits := Bit_Array.make_bit_array(70);

    Bit_Array.set_all_bits(*bits);
    Bit_Array.deinit(*bits);
    assert(bits.count == 0);
    assert(bits.slots.count == 0 && bits.slots.data == null);

    visited := 0;

    for bits visited += 1;

    assert(visited == 0);

    Bit_Array.init_bit_array(*bits, 10);
    Bit_Array.set_bit(*bits, 9);
    assert(bits.count == 10 && bits.slots.count == 1 && bits.slots[0] == 1 << 9);
    Bit_Array.deinit(*bits);
    print("ok\n");
}
