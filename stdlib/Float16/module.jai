// The 16-bit IEEE half-precision storage format, with conversions to and from `float32` and the `Math` vector and matrix types.
//
// A `Float16` holds only the bits (1 sign, 5 exponent, 10 significand) and has no arithmetic: convert to `float32` with [float16_to_float32], compute, and convert back with [float32_to_float16]. Halves have about three decimal digits and top out at 65504.
//
// Pass `ASSERT_ON_OVERFLOW = true` when importing to make a finite value too large for a half fail an assert instead of becoming infinity.
//
// ```jai
// half := float32_to_float16(1.5);
// assert(float16_to_float32(half) == 1.5);
// ```

#module_parameters(ASSERT_ON_OVERFLOW := false);

// A half-precision float stored as 16 bits. It is distinct from `u16`, so it cannot be mixed with integers by accident; `cast(u16) h` gives the raw bits and `cast(Float16) bits` builds one from them.
//
// ```jai
// one := float32_to_float16(1.0);
// assert(cast(u16) one == 0x3c00);
// assert(float16_to_float32(cast(Float16) 0x4000) == 2.0);
// ```
Float16 :: Impl.Float16;
// Two `Float16` components. Convert with [vector2_float32] and [vector2_float16].
//
// ```jai
// #import "Math";
// h := vector2_float16(xy(1, 2));
// assert(float16_to_float32(h.y) == 2);
// ```
Vector2_Float16 :: Impl.Vector2_Float16;
// Three `Float16` components. Convert with [vector3_float32] and [vector3_float16].
Vector3_Float16 :: Impl.Vector3_Float16;
// Four `Float16` components. Convert with [vector4_float32] and [vector4_float16].
Vector4_Float16 :: Impl.Vector4_Float16;
// A 3x3 matrix of `Float16` with the same fields as `Math.Matrix3` (`_11` to `_33`). Convert with [matrix3_float32] and [matrix3_float16].
Matrix3_Float16 :: Impl.Matrix3_Float16;
// Converts a half to `float32`. The conversion is exact, including subnormals, signed zero, infinities and NaN.
//
// ```jai
// assert(half_to_float_fast4(cast(Float16) 0x3c00) == 1.0);
// assert(half_to_float_fast4(cast(Float16) 0xc000) == -2.0);
// assert(half_to_float_fast4(cast(Float16) 0x7bff) == 65504.0);
// assert(half_to_float_fast4(cast(Float16) 0x0001) == 5.9604644775390625e-8); // smallest subnormal
// ```
half_to_float_fast4 :: Impl.half_to_float_fast4;
// Converts a `float32` to the nearest half, rounding ties to even.
//
// - From 65520 up the result is infinity (or a failed assert with `ASSERT_ON_OVERFLOW`).
// - Tiny values become subnormals, and ones too small for that become zero.
// - A NaN stays a NaN.
//
// ```jai
// assert(cast(u16) float_to_half_fast3(1.0) == 0x3c00);
// assert(cast(u16) float_to_half_fast3(-2.0) == 0xc000);
// assert(cast(u16) float_to_half_fast3(65504.0) == 0x7bff);
// assert(cast(u16) float_to_half_fast3(1.0 / 3.0) == 0x3555);
// assert(cast(u16) float_to_half_fast3(100000.0) == 0x7c00); // overflow: infinity
// assert(cast(u16) float_to_half_fast3(cast(float32) 1.00048828125) == 0x3c00); // tie rounds to even
// ```
float_to_half_fast3 :: Impl.float_to_half_fast3;
// Converts a half to `float32`; the usual name for `half_to_float_fast4`.
//
// ```jai
// assert(float16_to_float32(cast(Float16) 0x4000) == 2.0);
// ```
float16_to_float32 :: Impl.float16_to_float32;
// Converts a `float32` to the nearest half; the usual name for `float_to_half_fast3`.
//
// ```jai
// assert(cast(u16) float32_to_float16(-2.0) == 0xc000);
// ```
float32_to_float16 :: Impl.float32_to_float16;
// Converts a `Math.Matrix3` to half precision, element by element.
//
// ```jai
// #import "Math";
// m := Matrix3_Identity;
// h := matrix3_float16(m);
// assert(cast(u16) h._11 == 0x3c00 && cast(u16) h._12 == 0);
// assert(matrix3_float32(h)._22 == 1.0);
// ```
matrix3_float16 :: Impl.matrix3_float16;
// Converts a `Matrix3_Float16` back to a `Math.Matrix3`, exactly.
//
// ```jai
// #import "Math";
// h := matrix3_float16(Matrix3_Identity);
// assert(matrix3_float32(h)._11 == 1.0);
// ```
matrix3_float32 :: Impl.matrix3_float32;
// Converts a `Vector2_Float16` to a `Math.Vector2`, exactly.
//
// ```jai
// #import "Math";
// v := vector2_float32(vector2_float16(xy(3, 4)));
// assert(v.x == 3 && v.y == 4);
// ```
vector2_float32 :: Impl.vector2_float32;
// Converts a `Math.Vector2` to half precision.
//
// ```jai
// #import "Math";
// h := vector2_float16(xy(0.5, -4));
// v := vector2_float32(h);
// assert(v.x == 0.5 && v.y == -4);
// ```
vector2_float16 :: Impl.vector2_float16;
// Converts a `Vector3_Float16` to a `Math.Vector3`, exactly.
//
// ```jai
// #import "Math";
// v := vector3_float32(vector3_float16(xyz(1, 2, 3)));
// assert(v.z == 3);
// ```
vector3_float32 :: Impl.vector3_float32;
// Converts a `Math.Vector3` to half precision.
//
// ```jai
// #import "Math";
// h := vector3_float16(xyz(1, 2, 3));
// v := vector3_float32(h);
// assert(v.x == 1 && v.y == 2 && v.z == 3);
// ```
vector3_float16 :: Impl.vector3_float16;
// Converts a `Vector4_Float16` to a `Math.Vector4`, exactly.
//
// ```jai
// #import "Math";
// h := vector4_float16(xyzw(0.5, -2, 8, 70000));
// v := vector4_float32(h);
// assert(v.x == 0.5 && v.y == -2 && v.z == 8);
// assert(v.w > 1e30); // 70000 overflowed to infinity in half precision
// ```
vector4_float32 :: Impl.vector4_float32;
// Converts a `Math.Vector4` to half precision.
//
// ```jai
// #import "Math";
// h := vector4_float16(xyzw(1, 2, 3, 70000));
// assert(float16_to_float32(h.w) > 1e30); // 70000 overflows to infinity
// assert(float16_to_float32(h.y) == 2);
// ```
vector4_float16 :: Impl.vector4_float16;

#scope_file
Impl :: #import "internal/numerics/half"(ASSERT_ON_OVERFLOW = ASSERT_ON_OVERFLOW);
