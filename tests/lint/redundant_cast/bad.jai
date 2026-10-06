#import "Basic";

main :: () {
    x: s32 = 5;
    y := cast(s32) x;
    count: int = 3;
    z := cast(s64) count;
    w: u8 = 1;
    v: u8 = xx w;
    print("% % %\n", y, z, v);
}
