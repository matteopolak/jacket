// A call through a procedure value packs variadic arguments like a direct call: the value keeps
// the parameter names, defaults and `..` of the procedure or the type it comes from.
// rules: proc.17
#import "Basic";

report :: (code: int, fmt: string, args: ..Any) -> string {
    return tprint("%: %", code, tprint(fmt, ..args));
}

sum :: (base: int, step := 10, rest: ..int) -> int {
    total := base + step;
    for rest total += it;
    return total;
}

plain :: (base: int, rest: [] int) -> int {
    return base + rest.count;
}

callback := report;

main :: () {
    assert(callback(3, "missing %", "ERR") == "3: missing ERR");
    assert(callback(4, "none") == "4: none");

    s := sum;

    assert(s(1) == 11);
    assert(s(1, 2, 3, 4) == 10);
    assert(s(base = 5, rest = 7, 8) == 30);
    xs := int.[9, 9];

    assert(s(1, 2, ..xs) == 21);
    assert(s(step = 3, base = 4) == 7);

    // A declared variadic type makes a procedure with a view parameter callable the same way.
    p: (base: int, rest: ..int) -> int = plain;

    assert(p(2, 3, 4) == 4);
    assert(p(2) == 2);
}
