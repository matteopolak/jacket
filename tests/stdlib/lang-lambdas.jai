// Lambda expressions: typed from the expected procedure type, polymorphic when bound
// with `::`, inferred results through `$R`, and baked arguments.

apply :: (a: int, f: (int) -> int) -> int { return f(a); }

map_sum :: (array: [] $T, f: (T) -> $R) -> R {
    total: R;
    for array total += f(it);
    return total;
}

call_with :: (arg: $T, f: (T)) {
    f(arg);
}

counter: int;

lam :: (x) => x + 100;
add :: (a, b) => a + b;
add10 :: #bake_arguments add(a = 10);

main :: () -> int {
    if apply(5, (x) => x * 2) != 10 return 1;
    if apply(5, x => x + 1) != 6 return 2;
    if apply(5, (x: int) => x - 1) != 4 return 3;

    if lam(1) != 101 return 4;
    if add(2, 3) != 5 return 5;
    if add(1.5, 2.0) != 3.5 return 6;
    if add10(5) != 15 return 7;

    values := int.[1, 2, 3, 4];
    if map_sum(values, x => x * x) != 30 return 8;
    if map_sum(values, x => cast(float) x * 0.5) != 5.0 return 9;

    call_with(7, x => { counter += x; });
    if counter != 7 return 10;

    f: (int, int) -> int = (a, b) => a * b;
    if f(6, 7) != 42 return 11;

    zero :: () => 1000;
    if zero() != 1000 return 12;
    return 0;
}
