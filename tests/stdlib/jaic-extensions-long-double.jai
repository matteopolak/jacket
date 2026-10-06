// `Long_Double` from the Jaic_Extensions module: C's `long double` with the target's format.
// Where that is float64 (Apple arm64, MSVC) only the format-independent checks run; the CLI tests
// also run this file with `-target x86_64-linux-gnu` (x87) and `-target aarch64-linux-gnu`
// (binary128), where the interpreter's soft-float does the arithmetic.
#import "Basic";
#import "Jaic_Extensions";

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

    #if LONG_DOUBLE_IS_WIDE {
        assert(size_of(Long_Double) == 16 && align_of(Long_Double) == 16);
        assert(size_of(Holder) == 32 && cast(s64) *h.value - cast(s64) *h == 16);
        // Bits past float64's 53 survive arithmetic, comparison and integer conversion.
        tiny := power_of_half(60);
        assert(tiny == folded_tiny);
        assert(one + tiny != one && cast(float64) (one + tiny) == 1);
        odd: s64 = (1 << 60) + 1;
        assert(cast(s64) cast(Long_Double) odd == odd);
        #if CPU == .X64 {
            // x87: a 64-bit significand; one more bit rounds to even.
            assert(one + power_of_half(63) != one && one + power_of_half(64) == one);
        } else {
            // binary128: a 113-bit significand.
            assert(one + power_of_half(112) != one && one + power_of_half(113) == one);
        }
        print("ok wide\n");
    } else {
        assert(Long_Double == float64);
        print("ok\n");
    }
}
