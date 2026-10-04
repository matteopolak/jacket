// Regression tests for declarations, polymorphism, #modify, #bake_constants and operator overloads.

Vec :: struct { x, y: int; }

// Multi-value and mixed declarations.
pair :: () -> int, int { return 1, 2; }

// A body may shadow its parameters.
shadow :: (dest: int, value: int) -> int {
    dest := value + 1;
    return dest;
}

// `#this` inside a procedure is the procedure.
factorial :: (n: int) -> int {
    if n <= 1 return 1;
    return n * #this(n - 1);
}

// A named variadic argument takes the positional arguments that follow it.
count_all :: (s := "", f := 0.0, v: ..string) -> int { return v.count; }

// A parenthesized list is a procedure type; a polymorphic procedure can be passed to it.
apply :: (f: (int) -> int, x: int) -> int { return f(x); }
twice :: (f: (x: int) -> int, x: int) -> int { return f(f(x)); }
square :: (x: $T) -> T { return x * x; }

// #modify rewrites the inferred bindings.
widen :: (a: $T) -> T #modify { T = s64; return true; } { return a; }
sum_widened :: (a: [] $T) -> $R #modify {
    R = T;
    info := cast(*Type_Info) T;
    if info.type == .INTEGER {
        i := cast(*Type_Info_Integer) T;
        if i.runtime_size < 4 { if i.signed R = s32; else R = u32; }
    }
    return true;
} {
    result: R = 0;
    for a result += it;
    return result;
}

// Baking polymorphic type variables and struct arguments.
show :: (v: $T) -> T { return v; }
show_vec :: #bake_constants show(T = Vec);
Grid :: struct (M: int, N: int) { cells: [M][N] int; }
Grid5 :: #bake_arguments Grid(M = 5);
Defaulted :: struct (n := 3) { values: [n] int; }

// Operator overloads for compound assignment and address-of-index.
Bag :: struct { items: [4] int; }
operator *= :: (b: *Bag, k: int) { for 0..3 b.items[it] *= k; }
operator *[] :: (b: *Bag, i: int) -> *int { return *b.items[i]; }

// Overload sets can be aliased.
area :: (v: Vec) -> int { return v.x * v.y; }
area :: (a: int, b: int) -> int { return a * b; }
size_of_thing :: area;

main :: () -> int {
    a, b := 1, 2;
    if a != 1 || b != 2 return 1;
    c := 5;
    d, c=, e := 7, 8, 9;
    if d != 7 || c != 8 || e != 9 return 2;
    ok:, second:, c = true, 4, 6;
    if !ok || second != 4 || c != 6 return 3;
    x, y := pair();
    if x != 1 || y != 2 return 4;

    if shadow(1, 10) != 11 return 5;
    if factorial(5) != 120 return 6;
    if count_all(f = 1.5, s = "a", v = "b", "c", "d") != 3 return 7;
    if apply(square, 7) != 49 || twice(square, 3) != 81 return 8;

    small: s8 = 3;
    w := widen(small);
    if type_of(w) != s64 return 9;
    bytes := u8.[200, 100];
    if sum_widened(bytes) != 300 || type_of(sum_widened(bytes)) != u32 return 10;

    v := show_vec(Vec.{1, 2});
    if v.x != 1 return 11;
    g: Grid5(N = 2);
    if Grid5(N = 2).M != 5 || type_of(g) != Grid5(N = 2) return 12;
    dflt: Defaulted;
    if dflt.n != 3 || dflt.values.count != 3 return 13;

    bag: Bag;
    bag.items = .[1, 2, 3, 4];
    bag *= 10;
    p := *bag[2];
    <<p += 1;
    if bag.items[0] != 10 || bag.items[2] != 31 return 14;

    if size_of_thing(Vec.{3, 4}) != 12 || size_of_thing(2, 5) != 10 return 15;

    // Constants declared later in a block are visible earlier.
    if later(3) != 6 return 16;
    later :: (n: int) -> int { return n * 2; }
    return 0;
}
