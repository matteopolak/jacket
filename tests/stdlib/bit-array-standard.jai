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
    for bits {
        assert(it == (it_index < 130));
        seen += 1;
    }

    assert(seen == 130);
    selected: [..] int;
    for: only_set bits array_add(*selected, it_index);
    assert(selected.count == 130 && selected[0] == 0 && selected[64] == 64 && selected[129] == 129);
    array_free(selected);
    deinit(*bits);
    print("ok\n");
}
