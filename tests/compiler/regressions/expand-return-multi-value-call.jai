// `return two(x);` in an `#expand` macro with several results gives each result of the call to
// one of the macro's, as a procedure's `return` does; extra results of the call are dropped.
#import "Basic";

two :: (x: int) -> int, string {
    return x, "s";
}

three :: () -> int, string, float {
    return 1, "t", 2.5;
}

forward :: (x: int) -> int, string #expand {
    return two(x);
}

fewer :: () -> int, string #expand {
    return three();
}

first :: () -> int #expand {
    return two(7);
}

main :: () {
    a, b := forward(3);

    assert(a == 3 && b == "s");
    c, d := fewer();

    assert(c == 1 && d == "t");
    assert(first() == 7);
}
