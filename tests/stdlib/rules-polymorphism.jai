// rules: poly.1 poly.2 poly.3 poly.4 poly.5 poly.6 poly.7 poly.8 poly.11 poly.12 poly.13 poly.14
// Self-checking program for the claims in docs/language/polymorphism.md.
#import "Basic";

bigger :: (a: $T, b: T) -> T {
    if a > b return a;
    return b;
}

make :: ($T: Type, v: $U) -> T {
    return cast(T) v;
}

length_of :: (items: [] $T) -> int {
    return items.count;
}

apply :: (a: $A, f: (A) -> $R) -> R {
    return f(a);
}

twice :: ($N: int) -> int {
    return N * 2;
}

tag :: (x: $T) -> Type {
    return T;
}

weighted :: (a: int, b: int, c: int) -> int {
    return a * 100 + b * 10 + c;
}

weighted_b :: #bake_arguments weighted(b = 5);
weighted_ab :: #bake_arguments weighted(a = 1, b = 2);

Grid :: struct($T: Type, $N: int) {
    cells: [N] T;
}

IntGrid :: #bake_arguments Grid(T = int);

Holder :: struct(sample: $T) {
    value: T;
}

Fixed :: struct(shape: [$N] $T) {
    items: [N] T;
}

Entity :: struct {
    id: int;
}

Player :: struct {
    #as using base: Entity;
    score: int;
}

Rock :: struct {
    mass: int;
}

kind :: (x: $T/Entity) -> int {
    return 1;
}

kind :: (x: $T) -> int {
    return 2;
}

main :: () {
    // poly.2: $T inferred from arguments.
    assert(bigger(3, 4) == 4, "bigger ints");
    assert(bigger(1.5, 0.5) == 1.5, "bigger floats");
    #assert type_of(bigger(1.5, 0.5)) == float32;
    // poly.3: an explicit type argument.
    m := make(float, 3);
    #assert type_of(m) == float;
    assert(m == 3, "make(float, 3)");
    // poly.4: [] $T from an array argument.
    assert(length_of(int.[1, 2, 3]) == 3, "length_of int array");
    assert(length_of(string.["a"]) == 1, "length_of string array");
    // poly.5: $R from a lambda's result.
    assert(apply(4, x => x + 0.5) == 4.5, "apply infers R");
    // poly.6: a constant $N parameter.
    assert(twice(21) == 42, "twice(21)");
    #assert twice(4) == 8;

    // poly.1: same constant arguments share one instance, different ones do not.
    assert(tag(1) == int && tag(2) == int && tag("s") == string, "instances by type");
    pa: (int) -> Type = tag;
    pb: (int) -> Type = tag;
    assert(pa == pb, "the same argument types reuse one instance");
    a1: Grid(int, 2);
    a2: Grid(int, 2);
    a3: Grid(int, 3);
    #assert type_of(a1) == type_of(a2);
    #assert type_of(a1) != type_of(a3);

    // poly.7: baking fixes named parameters; the rest stay in order.
    assert(weighted_b(1, 2) == 152, "bake b");
    assert(weighted_ab(9) == 129, "bake a and b");

    // poly.8: baking a struct leaves one taking the remaining parameters.
    g: IntGrid(3);
    assert(g.cells.count == 3, "IntGrid(3) has 3 cells");
    #assert type_of(g.cells[0]) == int;
    // poly.11: instances of the baked struct are the origin's instances.
    #assert IntGrid(3) == Grid(int, 3);

    // poly.12: type variables declared inside a struct parameter's type.
    h: Holder(2.5);
    #assert type_of(h.value) == float32;
    fx: Fixed(int.[1, 2, 3, 4]);
    assert(fx.items.count == 4, "N bound from the array argument");
    #assert type_of(fx.items[0]) == int;

    // poly.13: $T/Entity matches the struct or one with it as an #as base.
    e: Entity;
    pl: Player;
    assert(kind(e) == 1 && kind(pl) == 1, "Entity restriction accepts Entity and #as Entity");
    // poly.14: a failed restriction lets another overload win.
    r: Rock;
    assert(kind(r) == 2, "Rock falls to the unrestricted overload");

    print("ok\n");
}
