#import "Basic";

square :: (x: int) -> int { return x * x; }

main :: () {
    p: *void = cast(*void) square;
    f := square;
    assert(f == p, "proc value equals its address");
    assert(p == f);
    assert(!(f != p));
    other: *void = null;
    assert(f != other);
}
