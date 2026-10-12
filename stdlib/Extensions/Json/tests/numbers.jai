// Numbers: exact integers, correctly rounded float parsing (checked against the shortest
// round-trip printer on random bit patterns, and against known hard cases) and shortest output.
//
//     jaic run stdlib/Extensions/Json/tests/numbers.jai

#import "Basic";
#import "Random";
#import "Extensions/Json";

failures := 0;

check :: (condition: bool, what: string, loc := #caller_location) {
    if condition return;
    print("FAIL (line %): %\n", loc.line_number, what);
    failures += 1;
}

parse64 :: (text: string) -> float64, bool {
    value, error := from_json(float64, text);

    return value, error.kind == .NONE;
}

parse32 :: (text: string) -> float32, bool {
    value, error := from_json(float32, text);

    return value, error.kind == .NONE;
}

bits64 :: (value: float64) -> u64 {
    return cast,force(u64) value;
}

bits32 :: (value: float32) -> u32 {
    return cast,force(u32) value;
}

text :: (value: float64) -> string {
    return to_json(value);
}

printing :: () {
    check(text(0.1) == "0.1", "0.1");
    check(text(0.1 + 0.2) == "0.30000000000000004", "0.1 + 0.2");
    check(text(1.0) == "1.0", "an integral float keeps a point");
    check(text(100.0) == "100.0", "100");
    check(text(123456789.0) == "123456789.0", "123456789");
    check(text(1e20) == "100000000000000000000.0", "1e20");
    check(text(1e21) == "1e21", "1e21");
    check(text(1.5e300) == "1.5e300", "1.5e300");
    check(text(1e-6) == "0.000001", "1e-6");
    check(text(1e-7) == "1e-7", "1e-7");
    check(text(1.25e-9) == "1.25e-9", "1.25e-9");
    check(text(-2.5) == "-2.5", "negative");
    check(text(0.0) == "0.0", "zero");
    check(text(-0.0) == "-0.0", "negative zero");
    check(text(5e-324) == "5e-324", "smallest subnormal");
    check(text(1.7976931348623157e308) == "1.7976931348623157e308", "largest");
    check(text(9007199254740993.0) == "9007199254740992.0", "2^53 + 1 rounds");

    one: float32 = 0.1;

    check(to_json(one) == "0.1", "float32 0.1 is short");
    check(to_json(cast(float32) 16777216) == "16777216.0", "float32 2^24");
    check(to_json(cast(float32) 3.4028235e38) == "3.4028235e38", "float32 max");
    check(to_json(cast(float32) 1.0e-45) == "1e-45", "float32 smallest");
    check(to_json(cast(float32) 0.3) == "0.3", "float32 0.3");

    zero: float64 = 0;

    check(to_json(zero / zero) == "null", "NaN is null");
    check(to_json(1.0 / zero) == "null" && to_json(-1.0 / zero) == "null", "infinity is null");
}

parsing :: () {
    cases := Parse_Case.[
        .{ "0", 0 },
        .{ "1", 1 },
        .{ "0.1", 0.1 },
        .{ "1e22", 1e22 },
        .{ "1e23", 1e23 },
        .{ "9007199254740993", 9007199254740992.0 },
        .{ "9007199254740995", 9007199254740996.0 },
        .{ "2.2250738585072011e-308", 2.2250738585072011e-308 },
        .{ "2.2250738585072014e-308", 2.2250738585072014e-308 },
        .{ "4.9e-324", 5e-324 },
        .{ "2.4703282292062327e-324", 0 },
        .{ "2.4703282292062328e-324", 5e-324 },
        .{ "1.7976931348623157e308", 1.7976931348623157e308 },
        .{ "123456789012345678901234567890", 1.2345678901234568e29 },
        .{ "0.000000000000000000000000000001", 1e-30 },
        .{ "1E5", 100000 },
        .{ "1e+5", 100000 },
        .{ "-1.5e-3", -0.0015 },
        .{ "5e-325", 0 },
        .{ "0.30000000000000004", 0.1 + 0.2 },
        .{ "8.41e21", 8.41e21 },
        .{ "7.2057594037927933e16", 7.2057594037927933e16 },
        .{ "1.00000000000000011102230246251565404236316680908203125", 1.0 },
        .{ "1.00000000000000011102230246251565404236316680908203126", 1.0000000000000002 },
    ];

    for cases {
        value, ok := parse64(it.text);
        check(ok && bits64(value) == bits64(it.expected), tprint("parse % gave %", it.text, value));
    }

    negative_zero, ok0 := parse64("-0");

    check(ok0 && bits64(negative_zero) == 0x8000000000000000, "-0 keeps its sign");
    _, ok := parse64("1e309");

    check(!ok, "overflow is an error");
    _, ok = parse64("1.7976931348623159e308");
    check(!ok, "just above the largest float64");

    value32, ok32 := parse32("16777217");

    check(ok32 && value32 == 16777216, "float32 rounds to even");
    value32, ok32 = parse32("0.1");
    check(ok32 && bits32(value32) == bits32(0.1), "float32 0.1");
    value32, ok32 = parse32("3.4028235e38");
    check(ok32 && bits32(value32) == 0x7f7fffff, "float32 max");
    _, ok32 = parse32("3.5e38");
    check(!ok32, "float32 overflow");
    value32, ok32 = parse32("1e-46");
    check(ok32 && value32 == 0, "float32 underflow");
    value32, ok32 = parse32("1.4e-45");
    check(ok32 && bits32(value32) == 1, "float32 smallest subnormal");
}

Parse_Case :: struct {
    text: string;
    expected: float64;
}

random_round_trips :: (count: int) {
    random_seed(20261012);

    bad := 0;

    for 1..count {
        bits := (cast(u64) random_get() << 32) ^ cast(u64) random_get();
        value := cast,force(float64) bits;

        if value != value || value - value != 0 continue;

        text := to_json(value);
        back, ok := parse64(text);

        if !ok || bits64(back) != bits {
            bad += 1;
            if bad < 5 print("float64 %: % -> %\n", bits, text, back);
        }
    }

    check(bad == 0, "random float64 values round trip");

    bad = 0;

    for 1..count {
        bits := cast,trunc(u32) random_get();
        value := cast,force(float32) bits;

        if value != value || value - value != 0 continue;

        text := to_json(value);
        back, ok := parse32(text);

        if !ok || bits32(back) != bits {
            bad += 1;
            if bad < 5 print("float32 %: % -> %\n", bits, text, back);
        }
    }

    check(bad == 0, "random float32 values round trip");

    // Short decimals: parse, print, and the text comes back (it is the shortest form).
    bad = 0;

    for 1..count {
        whole := cast(int) (random_get() % 100000);
        text := tprint("%.%%%", whole, random_get() % 10, random_get() % 10, random_get() % 10);
        value, ok := parse64(text);

        if !ok {
            bad += 1;
        } else {
            trimmed := to_json(value);

            while trimmed.count > 1 && trimmed[trimmed.count - 1] == #char "0" && trimmed[trimmed.count - 2] != #char "." trimmed.count -= 1;

            expected := text;
            while expected.count > 1 && expected[expected.count - 1] == #char "0" && expected[expected.count - 2] != #char "." expected.count -= 1;

            if trimmed != expected bad += 1;
        }
    }

    check(bad == 0, "short decimals print as they were written");
}

integers :: () {
    value, error := from_json(s64, "-9223372036854775808");

    check(error.kind == .NONE && value == -9223372036854775807 - 1, "s64 min");
    _, error = from_json(s64, "-9223372036854775809");
    check(error.kind == .OUT_OF_RANGE, "below s64 min");
    _, error = from_json(s64, "9223372036854775808");
    check(error.kind == .OUT_OF_RANGE, "above s64 max");

    big, error2 := from_json(u64, "18446744073709551615");

    check(error2.kind == .NONE && big == 18446744073709551615, "u64 max");
    _, error2 = from_json(u64, "18446744073709551616");
    check(error2.kind == .OUT_OF_RANGE, "above u64 max");
    _, error2 = from_json(u64, "-1");
    check(error2.kind == .OUT_OF_RANGE, "negative u64");
    zero, error3 := from_json(u8, "-0");

    check(error3.kind == .NONE && zero == 0, "-0 is 0");
    _, error3 = from_json(u8, "256");
    check(error3.kind == .OUT_OF_RANGE, "u8 range");
    _, error3 = from_json(s8, "-129");
    check(error3.kind == .OUT_OF_RANGE, "s8 range");
    _, error3 = from_json(s32, "1e3");
    check(error3.kind == .TYPE, "an exponent is not an integer");
    f, error4 := from_json(float64, "12");

    check(error4.kind == .NONE && f == 12, "an integer reads as a float");
    _, error4 = from_json(float64, "\"12\"");
    check(error4.kind == .TYPE, "a string is not a number");
    _, error4 = from_json(float64, "01");
    check(error4.kind != .NONE, "leading zero");
    _, error4 = from_json(float64, "1.");
    check(error4.kind != .NONE, "dangling point");
    _, error4 = from_json(float64, ".5");
    check(error4.kind != .NONE, "no leading digit");
    _, error4 = from_json(float64, "+1");
    check(error4.kind != .NONE, "plus sign");
    _, error4 = from_json(float64, "1e");
    check(error4.kind != .NONE, "dangling exponent");
}

// Set to true for the large random sweep (about 15 s with a release jaic in the interpreter);
// the default count keeps the test to a few seconds even with a debug jaic.
FULL_SWEEP :: false;

main :: () {
    for pass: 0..1 {
        json_use_simd = pass == 1;
        json_index_min_size = 0;

        printing();
        parsing();
        integers();

        // The number reader does not look at the text's bytes in bulk, so one pass is enough.
        if pass == 0 random_round_trips(ifx FULL_SWEEP then 20000 else 1500);
    }

    assert(failures == 0);
    print("ok\n");
}
