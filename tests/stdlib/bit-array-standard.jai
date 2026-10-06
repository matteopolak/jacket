// rules: opov.2 opov.9
#import "Basic";
#import "Bit_Array";

main :: () {
    bits: Bit_Array;
    init_bit_array(*bits, 130);
    set_bit(*bits, 0);
    set_bit(*bits, 64);
    bits[129] = true;
    assert(bits[64] && bits[129] && !bits[1]);
    assert(bits.slots.count == 3);
    count := 0;
    for: only_set bits count += 1;
    assert(count == 3);
    toggle_all_bits(*bits);
    assert(!bits[0] && bits[1]);
    set_all_bits(*bits);
    assert(bits.slots[2] == 3);
    seen := 0;
    for bits if it seen += 1;
    assert(seen == 130);
    deinit(*bits);
    print("ok\n");
}
