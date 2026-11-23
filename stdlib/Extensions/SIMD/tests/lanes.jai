// Lane arithmetic, comparisons and masks, reductions, selection, reversal, halves and bit counts on
// fixed vectors, each checked against the value worked out by hand.
#import "Basic";
SIMD :: #import "Extensions/SIMD";

failures := 0;

expect :: (condition: bool, what: string, loc := #caller_location) {
    if condition return;
    print("FAIL (line %): %\n", loc.line_number, what);
    failures += 1;
}

V :: SIMD.Vector(s32, 4);

same :: (v: V, expected: [4] s32, what: string, loc := #caller_location) {
    for 0..3 if v[it] != expected[it] {
        expect(false, tprint("% lane %: got %, expected %", what, it, v[it], expected[it]), loc);
        return;
    }
}

main :: () {
    items := s32.[3, -1, 4, 1];
    a := SIMD.load(V, items);
    b := SIMD.splat(V, 2);

    same(a + b, .[5, 1, 6, 3], "a + b");
    same(a * b, .[6, -2, 8, 2], "a * b");
    same(SIMD.min(a, b), .[2, -1, 2, 1], "min(a, b)");
    same(SIMD.max(a, b), .[3, 2, 4, 2], "max(a, b)");
    same(SIMD.reverse(a), .[1, 4, -1, 3], "reverse(a)");

    expect(SIMD.reduce_add(a) == 7, "reduce_add");
    expect(SIMD.reduce_min(a) == -1, "reduce_min");
    expect(SIMD.reduce_max(a) == 4, "reduce_max");

    // Masks: a < b in lanes 1 and 3.
    below := SIMD.lt(a, b);

    expect(SIMD.to_bits(below) == 0b1010, tprint("to_bits of a < b is %", SIMD.to_bits(below)));
    expect(SIMD.count_true(below) == 2, "count_true");
    expect(SIMD.first_true(below) == 1, "first_true");
    expect(SIMD.last_true(below) == 3, "last_true");
    expect(SIMD.any(below) && !SIMD.all(below) && !SIMD.none(below), "any, all, none on a mixed mask");

    same(SIMD.select(below, a, b), .[2, -1, 2, 1], "select picks a where the mask holds");

    // A mask from bits round-trips.
    rebuilt := SIMD.mask_from_bits(SIMD.Mask(s32, 4), 0b1010);

    expect(SIMD.to_bits(rebuilt) == 0b1010, "mask_from_bits round trip");

    // Equality with a scalar: only lane 3 holds 1.
    expect(SIMD.count_true(SIMD.eq(a, 1)) == 1 && SIMD.first_true(SIMD.eq(a, 1)) == 3, "eq with a scalar");
    expect(SIMD.count_true(SIMD.ge(a, 4)) == 1, "ge with a scalar counts lane 2");

    // Halves and combine.
    low := SIMD.low_half(a);
    high := SIMD.high_half(a);

    expect(low[0] == 3 && low[1] == -1 && high[0] == 4 && high[1] == 1, "low_half and high_half");
    joined := SIMD.combine(low, high);

    same(joined, .[3, -1, 4, 1], "combine of the halves");

    // Bit counts on u64.
    expect(SIMD.bit_count(0xF0F0) == 8, "bit_count");
    expect(SIMD.trailing_zeros(8) == 3, "trailing_zeros of 8");
    expect(SIMD.leading_zeros(1) == 63, "leading_zeros of 1");

    if failures > 0 {
        print("% failures\n", failures);
        exit(1);
    }

    print("ok\n");
}
