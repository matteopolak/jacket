// An enum literal `.NAME` only fits an enum parameter, so a struct overload next to an enum one is
// not an ambiguity.
#import "Basic";

Foo :: struct {
    x: int;
}

Kind :: enum {
    A;
    B;
}

which :: (k: Kind) -> int {
    return 1;
}

which :: (s: Foo) -> int {
    return 2;
}

pair :: (s: Foo, k: Kind) -> int {
    return 3;
}

pair :: (k: Kind, s: Foo) -> int {
    return 4;
}

main :: () {
    assert(which(.A) == 1);
    assert(which(.{ x = 1 }) == 2);
    assert(pair(.A, .{ x = 2 }) == 4);
    assert(pair(.{ x = 2 }, .B) == 3);
    print("ok\n");
}
