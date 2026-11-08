#import "Basic";

A :: OS == .WINDOWS;
B :: CPU == .ARM64;
C :: size_of(int) == 8;
D :: cast(u8) 3 == 3;
E :: cast(float32) 3 == 3;

main :: () {
    x: u8 = 3;
    assert(x == 3);
}
