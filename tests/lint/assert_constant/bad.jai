#import "Basic";

#assert true;

next :: () -> int {
    counter += 1;
    return counter;
}

counter: int;

#assert true "never fires";

main :: () {
    x := 3;

    assert(true);
    assert(true, "always holds");
    assert(true, "x is %", x);
    assert(true, "side effect %", next());
    if x > 5 assert(true);

    print("%\n", x);
}
