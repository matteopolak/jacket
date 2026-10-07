// Language-level conversions found while compiling the Way_to_Jai examples.
#import "Basic";

Mode :: enum {
    OFF;
    ON;
    AUTO;
}

using Flags :: enum_flags u8 {
    FLAG_A :: 1;
    FLAG_B :: 2;
}

Pair :: struct {
    x: float;
    y: float;
}

c_string_length :: (s: *u8) -> int {
    n := 0;
    while s[n] n += 1;
    return n;
}

// The 5-argument overload must not capture struct-literal arguments.
quad :: (a: float, b: float, c: float, d: float, tint: float) -> int {
    return 1;
}

quad :: (a: Pair, b: Pair, c: Pair, d: Pair, tint: float) -> int {
    return 2;
}

choose :: (flag: bool) -> int {
    return 10;
}

choose :: (m: Mode) -> int {
    return 20;
}

main :: () {
    // E.loose is integer-compatible.
    loose := cast(Mode.loose) Mode.AUTO;
    as_int: int = loose;
    assert(as_int == 2);
    back: Mode.loose = 1;
    assert(cast(Mode) back == .ON);

    // String literals convert to C strings.
    assert(c_string_length("hello") == 5);
    assert(c_string_length(cast(*u8) "abc") == 3);

    // string and []u8 share a layout.
    text := "bytes";
    view := cast([] u8) text;
    assert(view.count == 5 && view[0] == #char "b");
    again := cast(string) view;
    assert(again == "bytes");

    // Arrays and strings cast to bool as "non-empty".
    numbers := int.[1, 2, 3];
    empty: [] int;
    assert(cast(bool) numbers);
    assert(!cast(bool) empty);
    assert(cast(bool) "x" && !cast(bool) "");

    // Struct literal arguments pick the aggregate overload; inferred members the enum one.
    assert(quad(.{ 1, 2 }, .{ 3, 4 }, .{ 5, 6 }, .{ 7, 8 }, 1.0) == 2);
    assert(choose(.ON) == 20);

    // One value assigned to many targets; compound assignment applied to each.
    a, b: int;
    a, b = 5;
    assert(a == 5 && b == 5);
    a, b += 1;
    assert(a == 6 && b == 6);
    p: Pair;
    p.x, p.y = 42, 108;
    assert(p.x == 42 && p.y == 108);

    // An integer meeting an untyped float constant becomes a float.
    n := 7;
    half := n * 0.5;
    assert(half == 3.5);
    assert(type_of(half) == float);

    // ifx with an implicit then takes the interesting part of the condition.
    x := 7;
    assert((ifx x > 5 else 0) == 7);
    assert((ifx x else 1) == 7);
    zero := 0;
    assert((ifx zero else 1) == 1);

    // formatInt wraps any integer.
    big: u64 = 0xfedc_ba98_7654_3210;
    assert(tprint("%", formatInt(big, base = 16)) == "fedcba9876543210");

    // File-scope `using` of an enum brings its members into scope.
    both := FLAG_A | FLAG_B;
    assert(cast(u8) both == 3);
    print("ok\n");
}
