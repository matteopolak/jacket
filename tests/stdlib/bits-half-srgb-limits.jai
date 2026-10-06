// Bit_Operations, Float16, Srgb and Sloppy_Math at run time: every integer width, half-float
// rounding (ties to even, overflow, subnormals, NaN) with an exhaustive round trip, the sRGB
// curves on scalars and vectors, and the type limits.
#import "Basic";
#import "Math";
#import "Float16";
#import "Srgb";
Bits :: #import "Bit_Operations";
Sloppy :: #import "Sloppy_Math";

main :: () {
    bit_operations();
    half_floats();
    srgb();
    limits();
    print("ok\n");
}

bit_operations :: () {
    assert(Bits.byte_swap(cast(u16) 0x1234) == 0x3412 && Bits.byte_swap(cast(u8) 0xab) == 0xab);
    assert(Bits.byte_swap(cast(u64) 0x0102_0304_0506_0708) == 0x0807_0605_0403_0201);
    assert(Bits.byte_swap(cast(s32) -2) == cast,no_check(s32) 0xfeff_ffff, "signed swap: %", Bits.byte_swap(cast(s32) -2));
    assert(Bits.byte_swap(Bits.byte_swap(cast(s64) -123456789)) == -123456789);
    wide: u64 = 0x1122_3344_5566_7788;
    Bits.byte_swap_in_place(*wide);
    assert(wide == 0x8877_6655_4433_2211);
    odd: [3] u8 = .[1, 2, 3];
    Bits.byte_swap_in_place(*odd);
    assert(odd[0] == 3 && odd[1] == 2 && odd[2] == 1, "any type swaps its bytes");

    assert(Bits.bit_scan_forward(cast(u64) 0x8000_0000_0000_0000) == 64 && Bits.bit_scan_reverse(cast(u64) 0x8000_0000_0000_0000) == 64);
    assert(Bits.bit_scan_forward(cast(u64) 0) == 0 && Bits.bit_scan_reverse(cast(u64) 1) == 1 && Bits.bit_scan_forward(cast(u64) 0x10_0000_0000) == 37);
    assert(Bits.bit_scan_forward(cast(s32) -1) == 1 && Bits.bit_scan_reverse(cast(s32) -1) == 32 && Bits.bit_scan_reverse(cast(s16) 0x4000) == 15);
    assert(Bits.bit_scan_forward(cast(u8) 0x80) == 8 && Bits.bit_scan_reverse(cast(u8) 0x81) == 8 && Bits.bit_scan_forward(cast(u8) 0x81) == 1);
    assert(Bits.popcount(cast(u64) 0xffff_ffff_ffff_ffff) == 64 && Bits.popcount(cast(u64) 0) == 0 && Bits.popcount(cast(u16) 0xf0f0) == 8);
    assert(Bits.popcount(cast(s64) -1) == 64 && Bits.popcount(cast(s32) 0x7fff_ffff) == 31 && Bits.popcount(cast(u8) 0x55) == 4);
    for i: 0..63 {
        one := (cast(u64) 1) << cast(u64) i;
        assert(Bits.bit_scan_forward(one) == i + 1 && Bits.bit_scan_reverse(one) == i + 1 && Bits.popcount(one) == 1, "single bit %", i);
        assert(Bits.popcount(one - 1) == i, "low mask %", i);
    }

    x := Vector3.{ 1, 2, 3 };
    y := Vector3.{ 1, 2, 3 };
    z := Vector3.{ 1, 2, 4 };
    assert(Bits.bits_equal(*x, *y) && !Bits.bits_equal(*x, *z));
    assert(Bits.bits_equal(*x, *z, 8) && !Bits.bits_equal(*x, *z, 12), "a byte count compares a prefix");
    nan_a := FLOAT32_NAN;
    nan_b := FLOAT32_NAN;
    assert(Bits.bits_equal(*nan_a, *nan_b) && nan_a != nan_b, "bits_equal compares bits, not values");
    a := "abcdef";
    b := "abcxef";
    assert(Bits.first_different_byte(a.data, b.data, 6) == 3 && Bits.first_different_byte(a.data, a.data, 6) == 6 && Bits.first_different_byte(a.data, b.data, 0) == 0);
}

half_floats :: () {
    to_bits :: (f: float32) -> u16 {
        return cast(u16) float32_to_float16(f);
    }
    assert(to_bits(1) == 0x3c00 && to_bits(-2) == 0xc000 && to_bits(0.5) == 0x3800 && to_bits(65504) == 0x7bff);
    assert(to_bits(65519) == 0x7bff && to_bits(65520) == 0x7c00 && to_bits(1e10) == 0x7c00 && to_bits(-1e10) == 0xfc00, "overflow rounds to infinity past 65520");
    assert(to_bits(FLOAT32_INFINITY) == 0x7c00 && to_bits(-FLOAT32_INFINITY) == 0xfc00);
    assert((to_bits(FLOAT32_NAN) & 0x7c00) == 0x7c00 && (to_bits(FLOAT32_NAN) & 0x3ff) != 0, "NaN stays NaN");
    assert(to_bits(0.000000059604644775390625) == 0x0001, "smallest subnormal");
    assert(to_bits(0.0000000298023223876953125) == 0x0000, "half of the smallest subnormal ties to even (zero)");
    assert(to_bits(0.0000000447034835815429688) == 0x0001, "three quarters rounds up");
    assert(to_bits(0.00006103515625) == 0x0400, "smallest normal");
    assert(to_bits(1.0009765625) == 0x3c01 && to_bits(1.00048828125) == 0x3c00 && to_bits(1.00146484375) == 0x3c02, "ties to even");
    assert(to_bits(-0.0) == 0x8000 && to_bits(1e-10) == 0 && to_bits(-1e-10) == 0x8000);
    assert(float16_to_float32(cast(Float16) 0x3555) == 0.333251953125 && float16_to_float32(cast(Float16) 0xfbff) == -65504);
    assert(float_to_half_fast3(3) == float32_to_float16(3) && half_to_float_fast4(cast(Float16) 0x4200) == 3);

    // Every half value converts to float32 exactly and back to the same bits (NaNs stay NaN).
    for raw: 0..0xffff {
        half := cast(Float16) raw;
        f := float16_to_float32(half);
        exponent := (raw >> 10) & 31;
        fraction := raw & 1023;
        if exponent == 31 && fraction != 0 {
            assert(isnan(f), "half % is NaN", raw);
            continue;
        }
        back := cast(u16) float32_to_float16(f);
        assert(back == raw, "half % -> % -> %", raw, f, back);
    }

    v := vector3_float16(Vector3.{ 0.1, -1000, 65504 });
    restored := vector3_float32(v);
    assert(abs(restored.x - 0.1) < 0.0001 && restored.y == -1000 && restored.z == 65504, "vector round trip: %", restored);
    m := Matrix3.{ 1, 2, 3, 4, 5, 6, 7, 8, 0.25 };
    hm := matrix3_float16(m);
    assert(cast(u16) hm._33 == 0x3400 && cast(u16) hm.floats[0] == 0x3c00 && matrix3_float32(hm) == m);
}

srgb :: () {
    close :: (a: float, b: float64) -> bool {
        return abs(cast(float64) a - b) < 0.00001;
    }
    assert(srgb_to_linear(0.0) == 0 && close(srgb_to_linear(1.0), 1) && close(srgb_to_linear(0.5), 0.21404114048223255));
    assert(close(srgb_to_linear(0.04), 0.04 / 12.92) && close(linear_to_srgb(0.003), 0.003 * 12.92), "linear segments");
    assert(close(linear_to_srgb(0.5), 0.7353569830524495) && close(linear_to_srgb(1.0), 1) && linear_to_srgb(0.0) == 0);
    assert(srgb_to_linear_fast(0.3) == srgb_to_linear(0.3) && linear_to_srgb_fast(0.3) == linear_to_srgb(0.3));
    for i: 0..100 {
        c := cast(float) i / 100;
        assert(close(linear_to_srgb(srgb_to_linear(c)), c), "round trip %", c);
        if i assert(srgb_to_linear(c) > srgb_to_linear(c - 0.01), "monotonic at %", c);
    }
    color := Vector3.{ 0.5, 0.04, 1 };
    linear := srgb_to_linear(color);
    assert(close(linear.x, 0.21404114048223255) && close(linear.y, 0.04 / 12.92) && close(linear.z, 1));
    assert(srgb_to_linear_fast(color) == linear && linear_to_srgb_fast(linear) == linear_to_srgb(linear));
    back := linear_to_srgb(linear);
    assert(close(back.x, 0.5) && close(back.y, 0.04) && close(back.z, 1));
    with_alpha := Vector4.{ 0.5, 0.5, 0.5, 0.25 };
    linear4 := srgb_to_linear(with_alpha);
    assert(linear4.w == 0.25 && close(linear4.x, 0.21404114048223255), "alpha is not curved");
    assert(linear_to_srgb(linear4).w == 0.25 && srgb_to_linear_fast(with_alpha).w == 0.25 && linear_to_srgb_fast(with_alpha).w == 0.25);
    assert(srgb_to_linear_pow2(color) == Vector3.{ 0.25, 0.0016, 1 } && srgb_to_linear_pow2(with_alpha) == Vector4.{ 0.25, 0.25, 0.25, 0.25 });
    assert(linear_to_srgb_sqrt(Vector3.{ 0.25, 1, 0 }) == Vector3.{ 0.5, 1, 0 } && linear_to_srgb_sqrt(Vector4.{ 0.25, 0.25, 0.25, 0.81 }) == Vector4.{ 0.5, 0.5, 0.5, 0.81 });
}

limits :: () {
    assert(Sloppy.max_of_type(u8) == 255 && Sloppy.max_of_type(s8) == 127 && Sloppy.max_of_type(u16) == 65535 && Sloppy.max_of_type(s32) == 2147483647);
    assert(Sloppy.max_of_type(u32) == 0xffff_ffff && Sloppy.max_of_type(s64) == 0x7fff_ffff_ffff_ffff && Sloppy.max_of_type(u64) == 0xffff_ffff_ffff_ffff);
    assert(Sloppy.min_of_type(s8) == -128 && Sloppy.min_of_type(s16) == -32768 && Sloppy.min_of_type(s64) == -0x7fff_ffff_ffff_ffff - 1 && Sloppy.min_of_type(u64) == 0 && Sloppy.min_of_type(u8) == 0);
    assert(Sloppy.max_of_type(bool) && !Sloppy.min_of_type(bool));
    f32_max := Sloppy.max_of_type(float32);
    f64_max := Sloppy.max_of_type(float64);
    f32_min := Sloppy.min_of_type(float32);
    f64_min := Sloppy.min_of_type(float64);
    assert(float32_bits(f32_max) == 0x7f7f_ffff && float32_bits(f32_min) == 0x0080_0000, "float32 limits");
    assert(float64_bits(f64_max) == 0x7fef_ffff_ffff_ffff && float64_bits(f64_min) == 0x0010_0000_0000_0000, "float64 limits: % %", f64_max, f64_min);
    assert(type_of(f64_max) == float64 && type_of(f32_min) == float32);

    assert(Sloppy.is_approximately_zero(0.00005) && !Sloppy.is_approximately_zero(0.0002) && Sloppy.is_approximately_zero(-0.0001));
    assert(Sloppy.is_approximately_zero(0.5, epsilon = 1) && !Sloppy.is_approximately_zero(0.5, epsilon = 0.1));
    assert(Sloppy.values_are_close(1, 1.00005) && !Sloppy.values_are_close(1, 1.001) && Sloppy.values_are_close(1, 1.001, 0.01));
    assert(Sloppy.values_are_close(Vector2.{ 1, 2 }, Vector2.{ 1.00001, 2 }) && !Sloppy.values_are_close(Vector2.{ 1, 2 }, Vector2.{ 1, 2.1 }));
    assert(Sloppy.values_are_close(Vector3.{ 1, 2, 3 }, Vector3.{ 1, 2, 3.00001 }) && !Sloppy.values_are_close(Vector3.{ 1, 2, 3 }, Vector3.{ 1.5, 2, 3 }, 0.1));
    assert(Sloppy.DEFAULT_EPSILON == 0.0001);
}
