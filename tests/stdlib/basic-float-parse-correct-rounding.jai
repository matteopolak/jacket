// Basic's string_to_float64 and string_to_float are correctly rounded: pseudo-random float64 and
// float32 bit patterns survive a trip through Basic's shortest printing and back, and the hard
// cases (a halfway case of 800 digits, the subnormal boundaries, overflow, float32 values a
// double rounding would get wrong) land on the right bits.
#import "Basic";

State :: struct {
    value: u64;
}

next_random :: (state: *State) -> u64 {
    // xorshift64*
    x := state.value;
    x ^= x >> 12;
    x ^= x << 25;
    x ^= x >> 27;
    state.value = x;

    return x * 0x2545_f491_4f6c_dd1d;
}

// Set to true for the large random sweep (tens of seconds in the interpreter); the default is
// sized to take a few seconds even under a debug jaic.
FULL_SWEEP :: false;
SAMPLES :: #run ifx FULL_SWEEP then 100000 else 1000;

failures := 0;

expect64 :: (text: string, bits: u64) {
    value, ok, rest := string_to_float64(text);
    got := cast,force(u64) value;

    if !ok || rest.count != 0 || got != bits {
        print("string_to_float64 of % gave bits 0x% (ok %, rest `%`), expected 0x%\n", text, formatInt(got, base = 16), ok, rest, formatInt(bits, base = 16));
        failures += 1;
    }
}

expect32 :: (text: string, bits: u32) {
    value, ok, rest := string_to_float(text);
    got := cast,force(u32) value;

    if !ok || rest.count != 0 || got != bits {
        print("string_to_float of % gave bits 0x% (ok %, rest `%`), expected 0x%\n", text, formatInt(got, base = 16), ok, rest, formatInt(bits, base = 16));
        failures += 1;
    }
}

repeat_char :: (c: u8, count: s64) -> string {
    builder: String_Builder;

    for 1..count append(*builder, c);

    return builder_to_string(*builder);
}

hard_cases_64 :: () {
    expect64("2.2250738585072011e-308", 0x000f_ffff_ffff_ffff); // the largest subnormal
    expect64("2.2250738585072012e-308", 0x0010_0000_0000_0000); // the smallest normal
    expect64("2.2250738585072014e-308", 0x0010_0000_0000_0000);
    expect64("9007199254740993", 0x4340_0000_0000_0000); // 2^53 + 1 ties to even
    expect64("9007199254740995", 0x4340_0000_0000_0002); // 2^53 + 3 ties to even
    expect64("4.9e-324", 1);
    expect64("5e-324", 1);
    expect64("2.4703282292062327e-324", 0); // just below half the smallest subnormal
    expect64("2.4703282292062328e-324", 1);
    expect64("1e-400", 0);
    expect64("-1e-400", 0x8000_0000_0000_0000);
    expect64("1.7976931348623157e308", 0x7fef_ffff_ffff_ffff);
    expect64("1.7976931348623158e308", 0x7fef_ffff_ffff_ffff); // below the halfway point to 2^1024
    expect64("1.7976931348623159e308", 0x7ff0_0000_0000_0000);
    expect64("-1.7976931348623159e308", 0xfff0_0000_0000_0000);
    expect64("1e309", 0x7ff0_0000_0000_0000);
    expect64("0.0", 0);
    expect64("-0", 0x8000_0000_0000_0000);
    expect64("0.3", 0x3fd3_3333_3333_3333);

    // A halfway case 800 digits long: 2^53 + 1 followed by zeros ties to even; any nonzero
    // digit after it, even past the 800 digits that are kept, breaks the tie upwards.
    zeros := repeat_char(#char "0", 780);

    expect64(tprint("9007199254740993.%", zeros), 0x4340_0000_0000_0000);
    expect64(tprint("9007199254740993.%%", zeros, "1"), 0x4340_0000_0000_0001);
    expect64(tprint("9007199254740993.%%", repeat_char(#char "0", 1500), "1"), 0x4340_0000_0000_0001);

    // Many digits of an ordinary value: still the nearest double.
    expect64(tprint("1.%", repeat_char(#char "0", 5000)), 0x3ff0_0000_0000_0000);
    expect64(tprint("0.%%e301", repeat_char(#char "0", 300), "1"), 0x3ff0_0000_0000_0000);
}

hard_cases_32 :: () {
    // 1 + 2^-24 is halfway between 1 and the next float32.
    expect32("1.00000005960464477539062499", 0x3f80_0000);
    expect32("1.0000000596046447753906250", 0x3f80_0000);
    expect32("1.00000005960464477539062501", 0x3f80_0001);
    expect32("3.4028235677973366e38", 0x7f7f_ffff); // below halfway to 2^128
    expect32("3.4028235677973367e38", 0x7f80_0000);
    expect32("3.4028234663852886e38", 0x7f7f_ffff);
    expect32("16777217", 0x4b80_0000);
    expect32("16777217.0000000001", 0x4b80_0001);
    expect32("1.17549435e-38", 0x0080_0000);
    expect32("1.1754942e-38", 0x007f_ffff);
    expect32("1.4e-45", 1);
    expect32("7e-46", 0); // 7.0064923e-46 is half the smallest subnormal; this is below
    expect32("7.00649233e-46", 1);
    expect32("1e-60", 0);
    expect32("1e39", 0x7f80_0000);
    expect32("-1e39", 0xff80_0000);
    expect32("0.1", 0x3dcc_cccd);

    // Just above the float32 halfway point by far less than a float64 ulp: rounding through a
    // float64 first would land on the tie and go to even.
    expect32("1.000000059604644775390625000000000867361737988403547205962240695953369140625", 0x3f80_0001);
}

// The digits of a float64 magnitude (given as bits) in scientific form: `requested` significant
// digits, or the fewest that read back as the same float64 when it is -1.
spell :: (negative: bool, magnitude: u64, requested: s64) -> string {
    digits := __float_decimal_digits(magnitude, requested);
    builder: String_Builder;

    if negative append(*builder, #char "-");

    append(*builder, digits.digits[0] + #char "0");

    if digits.count > 1 {
        append(*builder, #char ".");
        for 1..digits.count - 1 append(*builder, digits.digits[it] + #char "0");
    }

    append(*builder, tprint("e%", digits.exponent));

    return builder_to_string(*builder);
}

check64 :: (bits: u64, requested: s64) {
    text := spell((bits >> 63) != 0, bits & 0x7fff_ffff_ffff_ffff, requested);
    back, ok, rest := string_to_float64(text);
    got := cast,force(u64) back;

    if !ok || rest.count != 0 || got != bits {
        print("float64 0x% spelled % read back as 0x%\n", formatInt(bits, base = 16), text, formatInt(got, base = 16));
        failures += 1;
    }
}

check32 :: (bits: u32, requested: s64) {
    // The float32 widened to a float64 is exact, so these digits describe it as precisely as asked.
    wide := cast(float64) (cast,force(float32) bits);
    wide_bits := cast,force(u64) wide;
    text := spell((bits >> 31) != 0, wide_bits & 0x7fff_ffff_ffff_ffff, requested);
    back, ok, rest := string_to_float(text);
    got := cast,force(u32) back;

    if !ok || rest.count != 0 || got != bits {
        print("float32 0x% spelled % read back as 0x%\n", formatInt(bits, base = 16), text, formatInt(got, base = 16));
        failures += 1;
    }
}

round_trips :: () {
    state := State.{ value = 0x9e37_79b9_7f4a_7c15 };

    for 1..SAMPLES {
        bits := next_random(*state);
        if (bits >> 52 & 0x7ff) == 0x7ff continue; // NaN and infinity have no number spelling

        check64(bits, -1);
        if it % 4 == 0 check64(bits, 40); // a long expansion is no harder to read
    }

    for 1..SAMPLES {
        bits := cast,trunc(u32) (next_random(*state) >> 20);
        if (bits >> 23 & 0xff) == 0xff continue;

        check32(bits, -1);
        check32(bits, 9);
    }

    // Subnormals and the edges of the exponent range specifically.
    for 1..SAMPLES / 5 {
        bits := next_random(*state) & 0x800f_ffff_ffff_ffff;
        if it % 2 == 0 bits |= 0x7fe0_0000_0000_0000 & next_random(*state);
        if (bits >> 52 & 0x7ff) == 0x7ff continue;

        check64(bits, -1);
    }

    for 1..SAMPLES / 5 {
        bits := cast,trunc(u32) (next_random(*state) >> 20) & 0x807f_ffff;
        if it % 2 == 0 bits |= 0x7f00_0000 & cast,trunc(u32) next_random(*state);
        if (bits >> 23 & 0xff) == 0xff continue;

        check32(bits, -1);
    }
}

main :: () {
    hard_cases_64();
    hard_cases_32();
    round_trips();

    assert(failures == 0, "% float conversions were not correctly rounded", failures);
}
