// Bit patterns of x87 extended-precision arithmetic on pseudo-random operands. The interpreter's
// soft-float output (`jaic run -target x86_64-...`) must match the hardware's (a native x86-64
// build) byte for byte.
#import "Basic";
#import "Jaic_Extensions";

state: u64 = 0x1234_5678_9abc_def1;
next :: () -> u64 {
    state ^= state << 13;
    state ^= state >> 7;
    state ^= state << 17;
    return state;
}

random_value :: () -> Long_Double {
    r := next();
    mantissa := cast(Long_Double) cast(s64) (r >> 11);
    k := cast(s64) (r & 63) - 32;
    scale: Long_Double = 1;
    two: Long_Double = 2;
    if k > 0 for 1..k scale *= two;
    if k < 0 for 1..-k scale /= two;
    v := mantissa * scale / 3;
    if r & 0x100 v = -v;
    return v;
}

dump :: (x: Long_Double) {
    bytes := cast(*u8) *x;
    for i: 0..9 print("%", formatInt(bytes[9 - i], base = 16, minimum_digits = 2));
    print(" ");
}

main :: () {
    acc: Long_Double = 1;
    for 1..300 {
        a := random_value();
        b := random_value();
        dump(a + b); dump(a - b); dump(a * b); dump(a / b);
        acc = acc * a / (b + 1) + a;
        dump(acc);
        print("% % % %|", a < b, a == b, cast(s64) a, cast(float64) b);
        if it % 4 == 0 print("\n");
    }
    print("\n");
}
