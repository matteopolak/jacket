#import "Basic";

#assert size_of(int) == 8;

// A macro may be expanded where the condition is not constant.
check :: (ok: bool) #expand {
    assert(true);
    assert(ok);
}

// Without the `@AssertsArgument` note, a procedure is not an assertion.
my_assert :: (ok: bool) {}

main :: () {
    x := 3;

    assert(x > 0);
    assert(false, "unreachable");
    assert(x == 3, "x is %", x);
    assert(!false);
    my_assert(true);
    check(x > 0);

    print("%\n", x);
}
