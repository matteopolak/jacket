// rules: must.3 must.4 must.5 must.9 must.10 must.12
// `#must` results and `#discard` parameters (see also must-return-values.jai,
// discard-parameters.jai and tests/corpus/negative/must-*.jai, discard-*.jai).
#import "Basic"()(ENABLE_ASSERT = false);

calls: int;

expensive :: () -> bool {
    calls += 1;
    return false;
}

side :: () -> int {
    calls += 1;
    return 4;
}

// must.5: `#must` is per result; here only the first one is marked.
first_must :: () -> int #must, bool {
    return 3, true;
}

only :: (x: int) -> int #must {
    return x + 1;
}

V :: struct {
    x: int;
}

operator + :: (a: V, b: V) -> V #must {
    return .{ a.x + b.x };
}

with_default :: (#discard v := side()) -> int {
    return 1;
}

typed :: (#discard x: $T) -> T {
    zero: T;
    return zero;
}

discarded_arg :: (#discard x: int) {}

main :: () {
    // must.4: with ENABLE_ASSERT off, Basic's assert neither evaluates nor checks its
    // condition.
    assert(expensive());
    assert(calls == 0);
    if calls != 0 exit(1);

    // must.5
    a := first_must();
    if a != 3 exit(2);

    // must.9: calls through a procedure value and operator overloads do not carry #must.
    p := only;
    p(1);
    u, w: V;
    u + w; // jailint: allow(no_effect) the discarded value is what is tested

    // must.3: the argument to a #discard parameter is not evaluated.
    discarded_arg(side());
    if calls != 0 exit(3);

    // must.10: a polymorphic #discard parameter still infers its type from the argument.
    f := typed(cast(float64) 2.5);
    if type_of(f) != float64 || f != 0 exit(4);
    s := typed("text");
    if type_of(s) != string exit(5);

    // must.12: a discarded parameter's default value is not evaluated either.
    if with_default() != 1 || calls != 0 exit(6);
    print("ok\n");
}
