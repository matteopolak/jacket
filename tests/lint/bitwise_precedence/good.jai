#import "Basic";

main :: () {
    n := 4;
    flags: u32 = 2;
    x := 0x1234;
    mask := (1 << n) - 1;
    set := flags | (1 << 3);
    low := (x & 0xFF) + 1;
    chained := x << 8 | n; // C reads it the same way
    both := (x >> 4) & 0xF;
    print("% % % % %\n", mask, set, low, chained, both);
}
