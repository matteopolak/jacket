// `Long_Double` from the Extensions/Long_Double module: C's `long double` with the target's format.
// Where that is float64 (Apple arm64, MSVC) only the format-independent checks run; the CLI tests
// also run this file with `-target x86_64-linux-gnu` (x87) and `-target aarch64-linux-gnu`
// (binary128), where the interpreter's soft-float does the arithmetic.
// rules: ext.4 ext.5 ext.6 ext.7 ext.9 ext.10 ext.11 ext.12 ext.13 ext.14 ext.15
#import "Basic";
#import "Extensions/Long_Double";
#import "Reflection";

THIRD :: cast(Long_Double) 1 / 3;
folded_tiny :: #run power_of_half(60);

power_of_half :: (n: s64) -> Long_Double {
    x: Long_Double = 1;
    for 1..n x /= 2;
    return x;
}

Holder :: struct {
    tag: u8;
    value: Long_Double;
}

scaled :: (x: Long_Double, k: Long_Double) -> Long_Double, bool {
    return x * k, x < k;
}

main :: () {
    // Format-independent behaviour.
    one: Long_Double = 1;
    third := one / 3;
    assert(third == THIRD);
    assert(cast(float64) third == 1.0 / 3.0);
    assert(tprint("%", third) == "0.333333");
    assert(tprint("%", type_of(third)) == "Long_Double" || !LONG_DOUBLE_IS_WIDE);
    h: Holder;
    h.value = 7;
    h.value += 0.5;
    h.value *= 2;
    assert(h.value == 15 && h.value > 14.75 && -h.value < 0);
    r, less := scaled(h.value, 2);
    assert(r == 30 && !less);
    mixed := h.value + cast(float64) 0.25;
    assert(mixed == 15.25 && type_of(mixed) == Long_Double);
    assert(cast(s32) mixed == 15 && cast(float32) mixed == 15.25);
    big: u64 = 0xffff_ffff_ffff_f800;
    assert(cast(u64) cast(Long_Double) big == big);
    info := type_info(Long_Double);
    assert(info.type == .FLOAT && info.runtime_size == size_of(Long_Double));
    zero: Long_Double;
    assert(-zero == zero);
    // NaN is unordered.
    nan := zero / zero;
    assert(nan != nan && !(nan == nan) && !(nan < one) && !(nan >= one));
    // Implicit conversions follow float64's: small integers, float32 and literals.
    small_int: s8 = -5;
    unsigned32: u32 = 7;
    single: float32 = 1.5;
    sum: Long_Double = small_int;
    sum += unsigned32;
    sum += single;
    assert(sum == 3.5 && cast(s16) sum == 3 && cast(float32) sum == 3.5);
    assert(cast(u64) (sum + 100.5) == 104 && cast(s64) -sum == -3);
    // print goes through float64: values beyond its range print as Inf or 0.
    huge: Long_Double = 1;
    for 1..1100 huge *= 2;
    tiny_value: Long_Double = 1;
    for 1..1100 tiny_value /= 2;
    assert(tprint("% % %", huge, -huge, tiny_value) == "Inf -Inf 0");

    #if LONG_DOUBLE_IS_WIDE {
        assert(size_of(Long_Double) == 16 && align_of(Long_Double) == 16);
        assert(size_of(Holder) == 32 && cast(s64) *h.value - cast(s64) *h == 16);
        // Bits past float64's 53 survive arithmetic, comparison and integer conversion.
        tiny := power_of_half(60);
        assert(tiny == folded_tiny);
        assert(one + tiny != one && cast(float64) (one + tiny) == 1);
        odd: s64 = (1 << 60) + 1;
        assert(cast(s64) cast(Long_Double) odd == odd);
        // A decimal literal is a float64 value first; arithmetic builds the exact value.
        from_literal := cast(Long_Double) 0.1;
        exact_tenth := cast(Long_Double) 1 / 10;
        assert(from_literal != exact_tenth && cast(float64) from_literal == 0.1 && cast(float64) exact_tenth == 0.1);
        // Out of s64's range: x87 gives the minimum (like fistp), binary128 saturates.
        beyond: Long_Double = 1;
        for 1..80 beyond *= 2;
        assert(cast(s64) -beyond == -9223372036854775808);
        // Reflection does not parse the wide type.
        parsed: Long_Double = 2;
        assert(!set_value_from_string(parsed, "1.5") && parsed == 2);
        #if CPU == .X64 {
            // x87: a 64-bit significand; one more bit rounds to even.
            assert(one + power_of_half(63) != one && one + power_of_half(64) == one);
            assert(cast(s64) beyond == -9223372036854775808);
        } else {
            // binary128: a 113-bit significand.
            assert(one + power_of_half(112) != one && one + power_of_half(113) == one);
            assert(cast(s64) beyond == 9223372036854775807);
        }
        print("ok wide\n");
    } else {
        assert(Long_Double == float64);
        print("ok\n");
    }
}
