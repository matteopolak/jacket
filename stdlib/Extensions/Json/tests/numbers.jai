// Numbers: exact integers, correctly rounded float parsing (checked against the shortest
// round-trip printer on random bit patterns, and against known hard cases) and shortest output.
//
//     jaic stdlib/Extensions/Json/tests/numbers.jai -interp

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

// The direct conversions (exact multiplication, Eisel-Lemire) must agree with native libc's
// decimal conversion, bit for bit, on every shape of token: random digits and exponents over the
// whole range (including subnormals and the edge of overflow), exact halfway cases between
// neighbouring floats, and neighbours of those.
libc :: #system_library "libc";
c_strtod :: (text: *u8, end: **u8) -> float64 #foreign libc "strtod";
c_strtof :: (text: *u8, end: **u8) -> float32 #foreign libc "strtof";

reference64 :: (text: string) -> u64, bool {
    value := c_strtod(temp_c_string(text), null);
    bits := bits64(value);

    return bits, ((bits >> 52) & 0x7ff) == 0x7ff;
}

reference32 :: (text: string) -> u32, bool {
    value := c_strtof(temp_c_string(text), null);
    bits := bits32(value);

    return bits, ((bits >> 23) & 0xff) == 0xff;
}

fast_path_agrees_with :: (text: string, bad: *int) {
    expected, overflow := reference64(text);
    value, ok := parse64(text);

    if ok == overflow || (ok && bits64(value) != expected) {
        bad.* += 1;
        if bad.* < 8 print("float64 % gave % (expected %, overflow %)\n", text, bits64(value), expected, overflow);
    }

    expected32, overflow32 := reference32(text);
    value32, ok32 := parse32(text);

    if ok32 == overflow32 || (ok32 && bits32(value32) != expected32) {
        bad.* += 1;
        if bad.* < 8 print("float32 % gave % (expected %, overflow %)\n", text, bits32(value32), expected32, overflow32);
    }
}

random_digits :: (builder: *String_Builder, count: int, zero_runs: bool) {
    for 1..count {
        if zero_runs && random_get() % 4 == 0 append(builder, "0");
        else if zero_runs && random_get() % 4 == 0 append(builder, "9");
        else print(builder, "%", random_get() % 10);
    }
}

fast_path_agrees :: (count: int) {
    random_seed(7771);

    bad := 0;

    for 1..count {
        builder: String_Builder;

        if random_get() % 8 == 0 append(*builder, "-");

        whole := cast(int) (random_get() % 20);
        fraction := cast(int) (random_get() % 20);
        zero_runs := random_get() % 3 == 0;

        if whole == 0 {
            append(*builder, "0");
        } else {
            print(*builder, "%", 1 + random_get() % 9);
            random_digits(*builder, whole - 1, zero_runs);
        }

        if whole == 0 && fraction == 0 fraction = 1;

        if fraction > 0 {
            append(*builder, ".");
            random_digits(*builder, fraction, zero_runs);
        }

        kind := random_get() % 4;

        if kind == 1 print(*builder, "e%", cast(int) (random_get() % 61) - 30);
        else if kind == 2 print(*builder, "e%", cast(int) (random_get() % 700) - 360);
        else if kind == 3 print(*builder, "E+%", random_get() % 330);

        text := builder_to_string(*builder);

        fast_path_agrees_with(text, *bad);
        free(text);
    }

    // Halfway between two floats: an odd multiple of half a unit in the last place, as integers
    // between 2^53 and 2^64 (exact in a u64), with the neighbours on either side.
    for 1..count / 4 {
        k := cast(u64) (random_get() % 11) + 1;
        m := (((cast(u64) random_get() << 32) | cast(u64) random_get()) & ((cast(u64) 1 << 52) - 1)) | (cast(u64) 1 << 52);
        halfway := (m << k) | ((cast(u64) 1) << (k - 1));

        for delta: -1..1 {
            text := tprint("%", halfway + cast(u64) delta);
            fast_path_agrees_with(text, *bad);
        }
    }

    // Floats near the ends of the range and around the subnormal boundary, written with their shortest digits.
    for 1..count / 4 {
        bits := (cast(u64) random_get() << 32) ^ cast(u64) random_get();
        pick := random_get() % 4;

        if pick == 0 bits = (bits & 0x000f_ffff_ffff_ffff) | ((cast(u64) (random_get() % 3)) << 52);
        else if pick == 1 bits = (bits & 0x000f_ffff_ffff_ffff) | ((cast(u64) (0x7fc + random_get() % 3)) << 52);
        else bits &= 0x7fff_ffff_ffff_ffff;

        value := cast,force(float64) bits;

        if value != value || value - value != 0 continue;

        fast_path_agrees_with(to_json(value), *bad);
    }

    check(bad == 0, "the direct float conversions agree with the exact one");
}

// Compare Json normal-number Schubfach digits with the independently implemented
// public Basic shortest formatter, preserving the digit/exponent assertions.
Expected_Digits :: struct {
    digits: [32] u8;
    count: int;
    exponent: int;
}

public_shortest_digits :: (value: float64) -> Expected_Digits {
    format: FormatFloat;
    format.value = value;
    format.mode = .SHORTEST;
    written := sprint("%", format);

    defer free(written);

    // Digits before any exponent, where the point is, and the exponent.
    digits: [32] u8;
    count := 0;
    point := -1;
    exponent := 0;
    i := 0;

    if written[0] == #char "-" i = 1;

    while i < written.count && written[i] != #char "e" {
        if written[i] == #char "." {
            point = count;
        } else {
            digits[count] = written[i] - #char "0";
            count += 1;
        }

        i += 1;
    }

    if point < 0 point = count;

    if i < written.count {
        i += 1;
        negative := written[i] == #char "-";

        if negative || written[i] == #char "+" i += 1;

        while i < written.count {
            exponent = exponent * 10 + written[i] - #char "0";
            i += 1;
        }

        if negative exponent = -exponent;
    }

    first := 0;

    while first < count - 1 && digits[first] == 0 {
        first += 1;
        point -= 1;
    }

    while count > first + 1 && digits[count - 1] == 0 count -= 1;

    count -= first;

    result: Expected_Digits;
    result.count = count;
    result.exponent = point - 1 + exponent;
    for 0..count - 1 result.digits[it] = digits[first + it];

    return result;
}

same_digits_as_exact :: (value: float64, bad: *int) {
    bits := bits64(value) & 0x7fff_ffff_ffff_ffff;

    if (bits >> 52) == 0x7ff || bits == 0 return;

    exact := public_shortest_digits(cast,force(float64) bits);
    written := to_json(value);

    // Digits before any exponent, where the point is, and the exponent.
    digits: [32] u8;
    count := 0;
    point := -1;
    exponent := 0;
    i := 0;

    if written[0] == #char "-" i = 1;

    while i < written.count && written[i] != #char "e" {
        if written[i] == #char "." {
            point = count;
        } else {
            digits[count] = written[i] - #char "0";
            count += 1;
        }

        i += 1;
    }

    if point < 0 point = count;

    if i < written.count {
        i += 1;
        negative := written[i] == #char "-";

        if negative || written[i] == #char "+" i += 1;

        while i < written.count {
            exponent = exponent * 10 + written[i] - #char "0";
            i += 1;
        }

        if negative exponent = -exponent;
    }

    first := 0;

    while first < count - 1 && digits[first] == 0 {
        first += 1;
        point -= 1;
    }

    while count > first + 1 && digits[count - 1] == 0 count -= 1;

    count -= first;

    matches := count == exact.count && point - 1 + exponent == exact.exponent;

    for 0..count - 1 {
        if matches && digits[first + it] != exact.digits[it] matches = false;
    }

    if !matches {
        bad.* += 1;
        if bad.* < 8 print("shortest digits of % (bits %): got %, exact has % digits, exponent %\n", written, bits, written, exact.count, exact.exponent);
    }
}

shortest_matches_exact :: (count: int) {
    random_seed(424242);

    bad := 0;

    // Every power of two and its neighbours: the irregular spacing and the integer fast path.
    for exponent: 0..2046 {
        // Every exponent in the full sweep; otherwise every 16th, and the ends of the range.
        if !FULL_SWEEP && exponent % 16 != 0 && exponent > 4 && exponent < 2042 continue;

        for delta: -2..2 {
            bits := (cast(u64) exponent << 52) + cast(u64) (delta + 2);
            same_digits_as_exact(cast,force(float64) bits, *bad);
            same_digits_as_exact(cast,force(float64) ((cast(u64) exponent << 52) - 1 - cast(u64) delta + 2), *bad);
        }
    }

    // The smallest subnormals and the largest numbers.
    for 1..ifx FULL_SWEEP then 2000 else 20 {
        same_digits_as_exact(cast,force(float64) cast(u64) it, *bad);
        same_digits_as_exact(cast,force(float64) (0x7fef_ffff_ffff_ffff - cast(u64) it), *bad);
    }

    // Integers and short decimals, as they usually appear.
    for 1..count {
        same_digits_as_exact(cast(float64) (cast(s64) random_get() - 2147483648), *bad);
        same_digits_as_exact(cast(float64) (cast(s64) random_get() % 100000) / 100.0, *bad);
        same_digits_as_exact(cast(float64) (cast(s64) ((cast(u64) random_get() << 21) ^ cast(u64) random_get())) * 1.0, *bad);
    }

    // Random bit patterns, and values with a random exponent in the usual range.
    for 1..count {
        bits := (cast(u64) random_get() << 32) ^ cast(u64) random_get();
        same_digits_as_exact(cast,force(float64) bits, *bad);

        small := cast(float64) random_get() / 4294967296.0 * 360.0 - 180.0;
        same_digits_as_exact(small, *bad);
    }

    check(bad == 0, "the printed digits are the exact shortest digits");
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
// the default counts keep the test well inside the runtime-test timeout even interpreted on a slow
// machine with a debug jaic.
FULL_SWEEP :: false;

main :: () {
    for pass: 0..1 {
        json_use_simd = pass == 1;
        json_index_min_size = 0;

        printing();
        parsing();
        integers();

        // The number reader does not look at the text's bytes in bulk, so one pass is enough.
        if pass == 0 random_round_trips(ifx FULL_SWEEP then 20000 else 1000);
        if pass == 0 shortest_matches_exact(ifx FULL_SWEEP then 300000 else 400);
        if pass == 0 fast_path_agrees(ifx FULL_SWEEP then 400000 else 600);
    }

    assert(failures == 0);
    print("ok\n");
}
