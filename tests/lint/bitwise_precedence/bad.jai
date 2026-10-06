#import "Basic";

main :: () {
    n := 4;
    flags: u32 = 2;
    x := 0x1234;
    mask := 1 << n - 1;
    set := flags | 1 << 3;
    low := x & 0xFF + 1;
    mixed := flags ^ flags & 4;
    print("% % % % %\n", mask, set, low, mixed, n * 2 >> 1);
}
