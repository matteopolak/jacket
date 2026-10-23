// A global initialized with a procedure literal holds a pointer to it and can be called and
// reassigned like any procedure-typed variable.
#import "Basic";

increment := (x: int) -> int {
    return x + 1;
};

double :: (x: int) -> int {
    return x * 2;
};

main :: () {
    assert(increment(2) == 3);

    increment = double;
    assert(increment(2) == 4);

    print("ok\n");
}
