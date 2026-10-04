#import "Basic";

// Passing a polymorphic lambda or an overload set to a polymorphic procedure-typed parameter.
map :: (array: [] $T, f: (T) -> $R) -> [..] R {
    result: [..] R;
    for array array_add(*result, f(it));
    return result;
}

square :: x => x * x;

twice :: (x: int) -> int { return x * 2; }
twice :: (s: string) -> string { return tprint("%%", s, s); }

varargs_named :: (s: string, i: int, v: ..Any) -> int { return i * 100 + v.count; }

main :: () {
    ints := int.[1, 2, 3];
    sq := map(ints, square);
    assert(sq.count == 3 && sq[2] == 9, "square");

    doubled := map(ints, twice);
    assert(doubled[1] == 4, "overload on int");

    strs := string.["a", "b"];
    ss := map(strs, twice);
    assert(ss[1] == "bb", "overload on string");

    // A named variadic keeps absorbing positional values until a later named argument.
    assert(varargs_named(s = "x", v = 1, "are", 2.0, i = 3) == 303);
}
