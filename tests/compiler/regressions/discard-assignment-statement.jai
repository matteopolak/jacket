// `_ = value;` evaluates the value and drops it.
#import "Basic";

calls := 0;

next :: () -> int {
    calls += 1;
    return calls;
}

main :: () {
    unused := 3;

    _ = unused;
    _ = next();
    _ = next() + 1;
    _ = "text";

    assert(calls == 2);

    print("ok\n");
}
