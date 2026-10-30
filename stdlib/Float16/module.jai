// The 16-bit IEEE half-precision storage format, with conversions to and from `float32` and the `Math` vector and matrix types.
//
// A `Float16` holds only the bits (1 sign, 5 exponent, 10 significand): it has no arithmetic of its own, so convert to `float32`, compute, and convert back. Half precision has about three decimal digits, a largest finite value of 65504, and subnormals down to 2^-24.
//
// The module parameter `ASSERT_ON_OVERFLOW` (default false) turns a finite value too large for a half, which otherwise becomes infinity, into a failed assert: `#import "Float16"(ASSERT_ON_OVERFLOW = true)`.

#module_parameters(ASSERT_ON_OVERFLOW := false);

// A half-precision float stored as its 16 bits. It is a distinct type based on `u16`, so a value cannot be mixed with integers or floats by accident; `cast(u16) h` gives the raw bits and `cast(Float16) bits` builds one from them.
//
// ```jai
// one := float32_to_float16(1.0);
// assert(cast(u16) one == 0x3c00);
// assert(float16_to_float32(cast(Float16) 0x4000) == 2.0);
// ```
Float16 :: Impl.Float16;
// Two `Float16` components. Convert with `vector2_float32` and `vector2_float16`.
Vector2_Float16 :: Impl.Vector2_Float16;
// Three `Float16` components. Convert with `vector3_float32` and `vector3_float16`.
Vector3_Float16 :: Impl.Vector3_Float16;
// Four `Float16` components. Convert with `vector4_float32` and `vector4_float16`.
Vector4_Float16 :: Impl.Vector4_Float16;
// A 3x3 matrix of `Float16`, with the same field names (`_11` to `_33`) and the same row-major `floats` overlay as `Math.Matrix3`. Convert with `matrix3_float32` and `matrix3_float16`.
Matrix3_Float16 :: Impl.Matrix3_Float16;
// Converts a half to `float32`. Every half value is exactly representable, so the conversion is exact: subnormals, signed zero and infinities carry over, and a NaN stays a NaN with its payload.
//
// Called `float16_to_float32` too.
//
// ```jai
// assert(half_to_float_fast4(cast(Float16) 0x3c00) == 1.0);
// assert(half_to_float_fast4(cast(Float16) 0xc000) == -2.0);
// assert(half_to_float_fast4(cast(Float16) 0x7bff) == 65504.0);
// assert(half_to_float_fast4(cast(Float16) 0x0001) == 5.9604644775390625e-8); // smallest subnormal
// ```
half_to_float_fast4 :: Impl.half_to_float_fast4;
// Converts a `float32` to the nearest half, rounding ties to the even significand. Called `float32_to_float16` too.
//
// - Values beyond the half range, from 65520 up, become infinity of the same sign (and fail an assert when `ASSERT_ON_OVERFLOW` is on). An infinity input stays an infinity without that assert.
// - Values below the smallest normal half (2^-14) become subnormals, and ones that round to less than the smallest subnormal become signed zero.
// - A NaN stays a NaN: the sign and the top bits of the payload are kept and the quiet bit is set, so a signalling NaN comes back quiet.
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
// The same procedure as `half_to_float_fast4`.
float16_to_float32 :: Impl.float16_to_float32;
// The same procedure as `float_to_half_fast3`.
float32_to_float16 :: Impl.float32_to_float16;
// Converts a `Math.Matrix3` to half precision, element by element in row-major order, rounding each as `float_to_half_fast3` does.
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
matrix3_float32 :: Impl.matrix3_float32;
// Converts a `Vector2_Float16` to a `Math.Vector2`, exactly.
vector2_float32 :: Impl.vector2_float32;
// Converts a `Math.Vector2` to half precision, rounding each component as `float_to_half_fast3` does.
//
// ```jai
// #import "Math";
// h := vector2_float16(xy(0.5, -4));
// v := vector2_float32(h);
// assert(v.x == 0.5 && v.y == -4);
// ```
vector2_float16 :: Impl.vector2_float16;
// Converts a `Vector3_Float16` to a `Math.Vector3`, exactly.
vector3_float32 :: Impl.vector3_float32;
// Converts a `Math.Vector3` to half precision, rounding each component as `float_to_half_fast3` does.
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
// Converts a `Math.Vector4` to half precision, rounding each component as `float_to_half_fast3` does.
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
