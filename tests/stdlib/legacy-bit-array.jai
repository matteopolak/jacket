#import "Basic";
Bits :: #import "legacy/Bit_Array";

main :: () -> s64 {
    bits := Bits.make_bit_array(130);
    defer Bits.deinit(*bits);
    if bits.slots.count != 3 || bits.count != 130 || !bits.allocator.proc return 1;
    Bits.set_bit(*bits, 63);
    Bits.set_bit_to(*bits, 64, true);
    Bits.toggle_bit(*bits, 129);
    if !bits[63] || !bits[64] || !bits[129] || bits[65] return 2;
    if bits.slots[0] != cast,no_check(s64)0x8000_0000_0000_0000 return 3;
    set_count := 0;
    for Bits.only_set(*bits) set_count += 1;
    if set_count != 3 return 4;
    unset_count := 0;
    for Bits.only_unset(*bits) unset_count += 1;
    if unset_count != 127 return 5;
    Bits.set_all_bits(*bits);
    if bits.slots[2] != -1 return 6;
    Bits.toggle_all_bits(*bits);
    for bit: bits { if bit return 7; }
    Bits.clear_bit(*bits, 63);
    Bits.clear_all_bits(*bits);
    for slot: bits.slots if slot != 0 return 8;
    empty := Bits.make_bit_array(0);
    defer Bits.deinit(*empty);
    if empty.slots.count != 0 return 9;
    return 0;
}
