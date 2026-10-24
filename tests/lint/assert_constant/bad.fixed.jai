#import "Basic";

next :: () -> int {
    counter += 1;
    return counter;
}

counter: int;

main :: () {
    x := 3;

    assert(true, "side effect %", next());
    if x > 5 assert(true);

    print("%\n", x);
}
