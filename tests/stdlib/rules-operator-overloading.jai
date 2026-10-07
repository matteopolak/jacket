// rules: opov.1 opov.2 opov.3 opov.4 opov.5 opov.6 opov.7 opov.9 opov.10 opov.11
// Self-checking program for the claims in docs/language/operator-overloading.md.
#import "Basic";
#import "Bit_Array";

V2 :: struct {
    x, y: float;
}

operator + :: (a: V2, b: V2) -> V2 {
    return .{ a.x + b.x, a.y + b.y };
}

operator == :: (a: V2, b: V2) -> bool {
    return a.x == b.x && a.y == b.y;
}

operator [] :: (v: V2, i: int) -> float {
    if i == 0 return v.x;
    return v.y;
}

operator - :: (a: V2) -> V2 {
    return .{ -a.x, -a.y };
}

operator * :: (a: V2, k: float) -> V2 {
    return .{ a.x * k, a.y * k };
}

operator * :: (a: V2, b: V2) -> float {
    return a.x * b.x + a.y * b.y;
}

Slots :: struct {
    cells: [3] int;
}

operator *[] :: (s: *Slots, i: int) -> *int {
    return *s.cells[i];
}

Doubler :: struct {
    stored: [4] int;
}

operator [] :: (d: Doubler, i: int) -> int {
    return d.stored[i];
}

operator []= :: (d: *Doubler, i: int, value: int) {
    d.stored[i] = value * 2;
}

Perm :: enum_flags u16 {
    READ :: 1;
    WRITE :: 2;
    EXEC :: 4;
}

main :: () {
    a := V2.{ 1, 2 };
    b := V2.{ 3, 4 };

    // opov.3: operator + on a struct.
    c := a + b;
    assert(c.x == 4 && c.y == 6, "V2 + V2");
    // opov.4: operator == on a struct.
    assert(!(a == b), "different V2 values are not ==");
    assert(c == V2.{ 4, 6 }, "equal V2 values are ==");
    // opov.5: operator [] on a struct value.
    assert(a[1] == 2 && a[0] == 1, "V2 indexing");

    // opov.1: binary and unary operators go through overload resolution by operand types.
    n := -a;
    assert(n.x == -1 && n.y == -2, "unary operator -");
    scaled := a * 2.0;
    assert(scaled.x == 2 && scaled.y == 4, "V2 * float picks the scalar overload");
    assert(a * b == 11.0, "V2 * V2 picks the dot overload");

    // opov.6: != falls back to !(==).
    assert(a != b, "!= from ==");
    assert(!(c != V2.{ 4, 6 }), "!= from == on equal values");

    // opov.7: x[i] with only operator *[] reads through the returned pointer. (Assigning
    // through it, opov.8, is pinned by index-operator-fallback.jai, which has no []= in scope.)
    s: Slots;
    s.cells[1] = 9;
    assert(s[1] == 9, "operator *[] dereferenced for reads");

    // opov.9: operator []= handles indexed assignment.
    d: Doubler;
    d[1] = 5;
    assert(d[1] == 10, "operator []= stores through the overload");

    // opov.2: a module's operator overloads apply where its type is used.
    bits: Bit_Array;
    init_bit_array(*bits, 16);
    bits[5] = true;
    assert(bits[5] && !bits[4], "Bit_Array's operator [] and []= are visible here");
    deinit(*bits);

    // opov.10: & | ~ on enum_flags keep the enum type.
    f: Perm = .READ;
    f |= .WRITE;
    both := f & .WRITE;
    #assert type_of(both) == Perm;
    #assert type_of(f | .EXEC) == Perm;
    #assert type_of(~f) == Perm;
    assert((f & ~ .READ) == .WRITE, "masking a flag out");
    // opov.11: a flag test compares with a literal zero.
    assert(f & .WRITE != 0, "set flag is nonzero");
    assert(f & .EXEC == 0, "unset flag is zero");

    print("ok\n");
}
