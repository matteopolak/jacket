#import "Basic";

main :: () {
    x: s32 = 5;
    y := x;
    count: int = 3;
    z := count;
    w: u8 = 1;
    v: u8 = w;
    print("% % %\n", y, z, v);
}
