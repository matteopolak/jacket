#import "Basic";
B :: #import "Bit_Operations";
S :: #import "Sort";
I :: #import "IntroSort";

main :: () {
    assert(B.byte_swap(cast(u32) 0x12345678) == 0x78563412);
    assert(B.bit_scan_forward(cast(u64) 0) == 0);
    assert(B.bit_scan_reverse(cast(u64) 0x8000000000000000) == 64);
    assert(B.popcount(cast(s8) -1) == 8);
    values: [7] s64 = .[9, -2, 0, 4, 9, 1, -7];
    result := S.quick_sort(values, (a, b) => a - b);

    assert(result.data == values.data && values[0] == -7 && values[6] == 9);
    I.intro_sort(values, (a, b) => b - a);
    assert(values[0] == 9 && values[6] == -7);
    print("ok\n");
}
