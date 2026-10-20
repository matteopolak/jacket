// Type variables and baked constants are inferred from many argument shapes: named `$T`
// arguments, baked variadics passed on, `Code` and defaulted constants, constant views,
// overloaded procedures for procedure-typed parameters, `null`, `#as` bases and the arguments
// of polymorphic structs.
#import "Basic";

ident :: (x: $T) -> T {
    return x;
}

count_names :: ($names: ..string) -> int {
    return names.count;
}

fwd :: ($names: ..string) -> int {
    return count_names(..names);
}

Query :: enum {
    A;
    B;
}

pick :: ($q: Query = .B) -> int {
    return cast(int) q;
}

calc :: ($c: Code) -> int #expand {
    return #insert c;
}

dflt :: (a: $T, $n := 3) -> int {
    return n;
}

len_of :: (a: [$N] $T) -> int {
    return N;
}

same_len :: (a: [$N] int, _b: [N] int) -> int {
    return N;
}

lit_len :: (a: [$N] int = .[1, 2]) -> int {
    return N;
}

withdef :: (a: [$N] int = .[1, 2, 3]) -> int {
    return N;
}

apply :: (f: (T) -> $R, x: $T) -> R {
    return f(x);
}

dbl :: (x: int) -> int {
    return x * 2;
}

dbl :: (x: float) -> float {
    return x * 2;
}

fixed_fn :: (f: ($A) -> $R) -> int {
    return 1;
}

sq :: (x: float) -> float {
    return x * x;
}

isnull :: (a: *$T, b: *T) -> bool {
    return a == b;
}

Base :: struct {
    id: int;
}

Derived :: struct {
    #as using b: Base;
    extra: int;
}

get_id :: (e: $E/Base) -> int {
    return e.id;
}

Pair :: struct($A: Type, $B: Type) {
    a: A;
    b: B;
}

first :: (p: Pair($X, $Y)) -> X {
    return p.a;
}

firstp :: (p: *Pair($X, $Y)) -> Y {
    return p.b;
}

anyp :: (_p: Pair) -> int {
    return 1;
}

tbl :: (p: $P/Pair) -> int {
    return 2;
}

second_named :: (p: Pair(B = $Y, A = $X)) -> Y {
    return p.b;
}

Wrapper :: struct {
    #as using inner: Pair(int, string);
}

via_as :: (p: Pair($X, $Y)) -> Y {
    return p.b;
}

via_as_ptr :: (p: *Pair($X, $Y)) -> X {
    return p.a;
}

run_lambda :: (x: $T, f: (T) -> T) -> T {
    return f(x);
}

cmp_is :: (a: $T, b: T, $cmp: (T, T) -> bool = null) -> bool {
    #if cmp return cmp(a, b);
    else return a == b;
}

lt :: (a: int, b: int) -> bool {
    return a < b;
}

Info :: struct {
    n: int;
}

baked_info :: ($info := Info.{}) -> int {
    return info.n;
}

baked_q :: ($q: Info) -> int {
    return q.n;
}

xs :: #run make_xs();

make_xs :: () -> [] int {
    a := NewArray(3, int);
    return a;
}

over_view :: ($v: [$N] int) -> int {
    return N;
}

variadic :: (xs: ..$T) -> int {
    return xs.count;
}

main :: () {
    assert(ident(5, T = s64) == 5);
    assert(ident(ifx true then 1 else 2) == 1);
    assert(ident(#char "a") == 97);
    assert(count_names("a", "b") == 2);
    assert(fwd("a", "b", "c") == 3);
    assert(pick() == 1 && pick(.A) == 0);
    x := 0;
    x = calc(x + 6);
    assert(x == 6);
    assert(dflt(1) == 3);
    arr: [4] float;

    assert(len_of(arr) == 4);
    assert(same_len(.[1, 2], .[3, 4]) == 2);
    assert(lit_len() == 2);
    assert(withdef() == 3);
    assert(apply(dbl, 3) == 6);
    assert(fixed_fn(sq) == 1);
    assert(!isnull(null, *x));
    d: Derived;
    d.b.id = 9;
    assert(get_id(d) == 9);
    p: Pair(int, string);
    p.a = 4;
    p.b = "hi";
    assert(first(p) == 4);
    assert(firstp(*p) == "hi");
    assert(anyp(p) == 1);
    assert(tbl(p) == 2);
    assert(tbl(*p) == 2);
    assert(second_named(p) == "hi");
    w: Wrapper;
    w.inner.b = "yo";
    assert(via_as(w) == "yo");
    assert(via_as_ptr(*w) == 0);
    assert(run_lambda(3, x => x + 1) == 4);
    assert(cmp_is(1, 2, lt));
    assert(cmp_is(1, 1));
    assert(baked_info() == 0);
    assert(baked_info(.{ n = 4 }) == 4);
    assert(baked_q(.{ n = 5 }) == 5);
    assert(over_view(xs) == 3);
    assert(variadic(1, 2, 3) == 3);
    nums := int.[4, 5];

    assert(variadic(..nums) == 2);
    print("ok\n");
}
