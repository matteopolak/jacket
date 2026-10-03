Bits :: #import "Bit_Operations";

main :: () -> int {
    word: u32 = 0x0123_4567;
    swapped := Bits.byte_swap(word);
    Bits.byte_swap_in_place(*word);
    if word != 0x6745_2301 || swapped != word return 1;
    if Bits.bit_scan_forward(cast(u32) 0) != 0 || Bits.bit_scan_reverse(cast(u32) 0) != 0 return 2;
    if Bits.bit_scan_forward(cast(u32) 8) != 4 || Bits.bit_scan_reverse(cast(u32) 9) != 4 return 3;
    if Bits.popcount(cast(s8) -1) != 8 return 4;
    return 0;
}
