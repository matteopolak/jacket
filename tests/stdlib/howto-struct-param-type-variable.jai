#import "Basic";

// A struct value parameter may declare a type variable: `struct (x: $T)`.

Thing :: struct(x: $T) {
    y := x;
}

Bling :: struct(x: [$N] $T) {
    y := x;
}

main :: () {
    a: Thing("Hello");
    assert(a.y == "Hello");
    assert(type_of(a.y) == string);

    b: Thing(42);
    assert(b.y == 42);
    assert(type_of(b.y) == s64);

    c: Bling(int.[1, 2, 3]);
    assert(c.y.count == 3 && c.y[2] == 3);
}
