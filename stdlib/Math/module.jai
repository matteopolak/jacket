// Vectors, quaternions, matrices, planes and the scalar math functions they build on.
//
// Conventions, used throughout:
// - Vectors, quaternions and matrices hold `float` (32-bit) components. A quaternion stores `x, y, z` (the vector part) and `w` (the scalar part); `Quaternion.{}` is the identity rotation.
// - Coordinate systems are right-handed and angles are in radians, counter-clockwise looking down the axis towards the origin.
// - Matrices are stored row-major: `_rc` is row `r`, column `c`, `v[r]` is row `r` as a vector and `coef[r][c]` and `floats[r * columns + c]` index the same storage. Vectors are columns, so `m * v` transforms `v`, and `a * b` applies `b` first. A transform's translation sits in the last column (`_14`, `_24`, `_34`).
// - Vector operators `+ - * /` and `==` work component by component, `vector * float` scales, and `float / vector` divides the scalar by each component. Quaternion `*` is the Hamilton product; matrix `*` is the matrix product, and `+`, `-` and `* float` are element-wise.
// - Float edge cases follow IEEE 754: NaN compares unequal to everything, so `==` on a vector holding a NaN is false.
//
// ```jai
// a := xyz(1, 2, 3);
// b := xyz(4, 5, 6);
// assert(a + b == xyz(5, 7, 9));
// assert(dot(a, b) == 32);
// assert(cross(xyz(1, 0, 0), xyz(0, 1, 0)) == xyz(0, 0, 1));
// assert(length(xyz(3, 4, 0)) == 5);
//
// m := make_translation_matrix4(xyz(10, 0, 0));
// assert(transform_point(m, xyz(1, 2, 3)) == xyz(11, 2, 3));
// ```

// A 2D vector with fields `x` and `y`, also reachable as the array `component`. Supports `+ - * /` (component-wise or by a scalar), unary `-`, `==`, and the procedures `dot`, `length`, `distance`, `lerp`, `min`, `max`, `abs`, `unit_vector` and `rotate`.
//
// ```jai
// v := Vector2.{ 3, 4 };
// assert(v.x == 3 && v.component[1] == 4);
// assert(length(v) == 5);
// assert(v * 2 == xy(6, 8));
// ```
Vector2 :: Impl.Vector2;
// A 3D vector with fields `x`, `y`, `z`, also reachable as the array `component`; `xy` and `yz` view two of the fields as a `Vector2`. Supports `+ - * /` (component-wise or by a scalar, including a `float64` scalar that is narrowed to `float`), unary `-`, `==`, and the procedures `dot`, `cross`, `length`, `distance`, `lerp`, `min`, `max`, `abs`, `unit_vector` and `rotate` (by a quaternion).
//
// ```jai
// v := Vector3.{ 1, 2, 3 };
// assert(v.xy == xy(1, 2) && v.yz == xy(2, 3));
// assert(v / 2 == xyz(0.5, 1, 1.5));
// assert(-v == xyz(-1, -2, -3));
// ```
Vector3 :: Impl.Vector3;
// A 4D vector with fields `x`, `y`, `z`, `w`, also reachable as the array `component`; `xy`, `yz`, `zw` view pairs as a `Vector2` and `xyz`, `yzw` triples as a `Vector3`. It is also the homogeneous coordinate type that `Matrix4` multiplies. Supports the same operators and procedures as `Vector3` except `cross` and `rotate`.
//
// ```jai
// v := Vector4.{ 1, 2, 3, 4 };
// assert(v.xyz == xyz(1, 2, 3) && v.zw == xy(3, 4));
// assert(dot(v, v) == 30);
// ```
Vector4 :: Impl.Vector4;
// A rotation as `x, y, z, w`, with the vector part in `xyz` and the scalar part last. A default `Quaternion.{}` is the identity (`w` is 1). A unit quaternion rotates; the procedures that expect one (`rotate`, `set_rotation`) do not normalize it, so call `normalize_or_identity` after arithmetic that drifts.
//
// `a * b` is the Hamilton product and applies `b`'s rotation first. Also supports `+`, `-`, unary `-`, `* float` and `==`, which act on the four components.
//
// ```jai
// q: Quaternion;
// set_from_axis_and_angle(*q, xyz(0, 0, 1), PI / 2); // quarter turn about z
// v := rotate(xyz(1, 0, 0), q);
// assert(abs(v.x) < 0.0001 && abs(v.y - 1) < 0.0001);
// assert(Quaternion.{} == Quaternion.{ 0, 0, 0, 1 });
// ```
Quaternion :: Impl.Quaternion;
// A complex number as a `Vector2` (`x` real, `y` imaginary), declared `#type,isa Vector2`: a distinct type that converts to `Vector2` where one is expected, so `length`, `dot` and the vector operators accept it. The module defines no complex multiplication or division of its own.
//
// ```jai
// c: Complex;
// c.x = 3;
// c.y = 4;
// assert(length(c) == 5);
// ```
Complex :: Impl.Complex;
// A plane `a*x + b*y + c*z + d = 0` stored as `a, b, c, d`; `normal` is the `(a, b, c)` part and `vector4` the four coefficients. `plane_dot(plane, point)` is the signed distance from the point to the plane when `normal` has length 1, positive on the side the normal points to.
//
// ```jai
// p := make_plane(xyz(0, 1, 0), xyz(0, 2, 0)); // the plane y = 2
// assert(p.d == -2 && p.normal == xyz(0, 1, 0));
// assert(plane_dot(p, xyz(5, 3, 5)) == 1);
// ```
Plane3 :: Impl.Plane3;
// A plain two-float struct with `x` and `y`, used as an interface type: a procedure taking `$T/interface AnyVector2` accepts any struct with those fields, not just `Vector2`.
AnyVector2 :: Impl.AnyVector2;
// A 2x2 matrix, row-major: `_11, _12, _21, _22`, also `v[row]` as `Vector2` rows, `coef[row][col]` and `floats[4]`. `Matrix2 * Vector2` multiplies by a column vector, and `Matrix2 * Matrix2` is the matrix product (`a * b` applies `b` first).
//
// ```jai
// m := Matrix2.{ 0, -1, 1, 0 }; // quarter turn counter-clockwise
// assert(m * xy(1, 0) == xy(0, 1));
// assert(m * m == Matrix2.{ -1, 0, 0, -1 });
// ```
Matrix2 :: Impl.Matrix2;
// The 2x2 identity: ones on the diagonal, zeros elsewhere.
Matrix2_Identity :: Impl.Matrix2_Identity;
// A plain struct of the four fields `_11` to `_22`, used as an interface type for procedures that accept any 2x2 matrix.
AnyMatrix2 :: Impl.AnyMatrix2;
// A 3x3 matrix, row-major: `_11` to `_33`, also `v[row]` as `Vector3` rows, `coef[row][col]` and `floats[9]`. `Matrix3 * Vector3` multiplies by a column vector, `Matrix3 * Matrix3` is the product, and `+`, `-` and `* float` work element by element. A rotation matrix of a quaternion is made by `rotation_matrix(Matrix3, q)`.
//
// ```jai
// m := Matrix3_Identity;
// m._13 = 5; // row 1, column 3
// assert(m.v[0] == xyz(1, 0, 5) && m.coef[0][2] == 5 && m.floats[2] == 5);
// assert(m * xyz(1, 1, 1) == xyz(6, 1, 1));
// ```
Matrix3 :: Impl.Matrix3;
// The 3x3 identity: ones on the diagonal, zeros elsewhere.
Matrix3_Identity :: Impl.Matrix3_Identity;
// A plain struct of the nine fields `_11` to `_33`, used as an interface type: `transform_vector`, `scale`, `set_rotation` and others accept any struct with those fields, which includes `Matrix4`.
AnyMatrix3 :: Impl.AnyMatrix3;
// A 3x4 affine transform, row-major: three rows of four (`_11` to `_34`), so there is no bottom row and the transform is a rotation or scale part in the left 3x3 and a translation in `_14`, `_24`, `_34`. Rows are `Vector4`, columns have three entries. `Matrix4x3 * Vector4` gives a `Vector3`, and `Matrix4x3 * Matrix4x3` composes two transforms.
//
// ```jai
// m := Matrix4x3_Identity;
// m._14 = 7;
// assert(m * xyzw(1, 2, 3, 1) == xyz(8, 2, 3)); // w = 1 picks up the translation
// assert(m * xyzw(1, 2, 3, 0) == xyz(1, 2, 3)); // w = 0 is a direction
// ```
Matrix4x3 :: Impl.Matrix4x3;
// The affine identity: ones on the diagonal and a zero translation.
Matrix4x3_Identity :: Impl.Matrix4x3_Identity;
// A plain struct of the twelve fields `_11` to `_34`, used as an interface type: `transform_point` accepts any struct with those fields, which includes `Matrix4`.
AnyMatrix4x3 :: Impl.AnyMatrix4x3;
// A 4x4 matrix, row-major: `_11` to `_44`, also `v[row]` as `Vector4` rows, `coef[row][col]`, `floats[16]` and `mat4x3` for the top three rows. It is the type for model, view and projection transforms: vectors are columns, `m * v` transforms `v`, `a * b` applies `b` first, and the translation lives in `_14`, `_24`, `_34`. `Matrix4 * Vector4` gives a `Vector4`.
//
// Upload it to a graphics API as it is: the 16 floats in memory are row 1 first, which a column-major API reads as the transpose, so transpose it or tell the API the data is row-major.
//
// ```jai
// t := make_translation_matrix4(xyz(1, 2, 3));
// s := make_scale_matrix4(xyz(2, 2, 2));
// m := t * s; // scale first, then translate
// assert(m * xyzw(1, 1, 1, 1) == xyzw(3, 4, 5, 1));
// assert(m.v[0] == xyzw(2, 0, 0, 1));
// ```
Matrix4 :: Impl.Matrix4;
// The 4x4 identity: ones on the diagonal, zeros elsewhere.
Matrix4_Identity :: Impl.Matrix4_Identity;
// A marker struct that matches every matrix type of this module (`Matrix2`, `Matrix3`, `Matrix4x3`, `Matrix4`); use it as `$T/interface AnyMatrix` to write one procedure for all of them, which can then rely on `floats` and `coef`.
AnyMatrix :: Impl.AnyMatrix;
// The circle constant 2*PI as a `float`, rounded to 7 digits (6.283185).
TAU :: Impl.TAU;
// The circle constant 2*PI at `float64` precision.
TAU64 :: Impl.TAU64;
// Pi as a `float` (3.1415927). Use `PI64` for double precision.
PI :: Impl.PI;
// Pi as a `float64` (3.141592653589793).
PI64 :: Impl.PI64;
// The square root of 2 as an untyped floating constant (about 1.41421356).
SQRT2 :: Impl.SQRT2;
// One over the square root of 2 (about 0.70710678), the square root of one half.
SQRTH :: Impl.SQRTH;
// The first of three pieces `DP1 + DP2 + DP3` that together hold pi/4 to more precision than a `float64`, so that a multiple of pi/4 can be subtracted from a large angle without losing its low bits (Cody-Waite argument reduction). Only useful when writing trigonometric functions.
DP1 :: Impl.DP1;
// The second piece of pi/4 for argument reduction; see `DP1`.
DP2 :: Impl.DP2;
// The third piece of pi/4 for argument reduction; see `DP1`.
DP3 :: Impl.DP3;
// An angle magnitude of about 1.07e9 (2^30) beyond which a trigonometric function written with `DP1` to `DP3` reduction has lost every significant digit. `sin` and `cos` here do not consult it; it is kept for code that does.
LOSSTH :: Impl.LOSSTH;
// `1.000001`: how far outside [-1, 1] an `asin` or `acos` argument could be before older code reported an error. `asin` and `acos` here return NaN for any argument beyond +/-1 and do not use it.
ASIN_AND_ACOS_ERROR_REPORTING_LIMIT :: Impl.ASIN_AND_ACOS_ERROR_REPORTING_LIMIT;
// The largest finite half-precision value, 65504, as a `float`.
FLOAT16_MAX :: Impl.FLOAT16_MAX;
// The smallest positive normal `float`, about 1.1755e-38. It is not the most negative float: use `-FLOAT32_MAX` for that.
FLOAT32_MIN :: Impl.FLOAT32_MIN;
// The largest finite `float`, about 3.4028235e38.
FLOAT32_MAX :: Impl.FLOAT32_MAX;
// Positive infinity as a `float`. Negate it for negative infinity.
FLOAT32_INFINITY :: Impl.FLOAT32_INFINITY;
// A quiet-or-signalling NaN (bit pattern `0x7FBFFFFF`) as a `float`. It compares unequal to every value including itself; test for it with `isnan`, not `==`.
FLOAT32_NAN :: Impl.FLOAT32_NAN;
// The smallest positive normal `float64`, about 2.2250738585072014e-308. It is not the most negative double: use `-FLOAT64_MAX`.
FLOAT64_MIN :: Impl.FLOAT64_MIN;
// The largest finite `float64`, about 1.7976931348623157e308.
FLOAT64_MAX :: Impl.FLOAT64_MAX;
// Positive infinity as a `float64`.
FLOAT64_INFINITY :: Impl.FLOAT64_INFINITY;
// A NaN (bit pattern `0x7FF7FFFFFFFFFFFF`) as a `float64`; test for it with `isnan`.
FLOAT64_NAN :: Impl.FLOAT64_NAN;
// The smallest `s8`, -128. The `S*_MIN`, `S*_MAX` and `U*_MAX` constants are typed, so they can be used where the matching integer type is expected without a cast.
S8_MIN :: Impl.S8_MIN;
// The largest `s8`, 127.
S8_MAX :: Impl.S8_MAX;
// The largest `u8`, 255.
U8_MAX :: Impl.U8_MAX;
// The smallest `s16`, -32768.
S16_MIN :: Impl.S16_MIN;
// The largest `s16`, 32767.
S16_MAX :: Impl.S16_MAX;
// The largest `u16`, 65535.
U16_MAX :: Impl.U16_MAX;
// The smallest `s32`, -2147483648.
S32_MIN :: Impl.S32_MIN;
// The largest `s32`, 2147483647.
S32_MAX :: Impl.S32_MAX;
// The largest `u32`, 4294967295.
U32_MAX :: Impl.U32_MAX;
// The smallest `s64`, -9223372036854775808.
S64_MIN :: Impl.S64_MIN;
// The largest `s64`, 9223372036854775807.
S64_MAX :: Impl.S64_MAX;
// The largest `u64`, 18446744073709551615.
U64_MAX :: Impl.U64_MAX;
// The smallest whole number not below `x`, as a `float` or `float64`. Infinities, NaN and values already whole are returned as they are.
//
// ```jai
// assert(ceil(1.2) == 2 && ceil(-1.8) == -1 && ceil(3.0) == 3);
// ```
ceil :: Impl.ceil;
// The largest whole number not above `x`, as a `float` or `float64`. It rounds toward negative infinity, so `floor(-1.2)` is -2. Infinities, NaN and values already whole are returned as they are.
//
// ```jai
// assert(floor(1.8) == 1 && floor(-1.2) == -2 && floor(3.0) == 3);
// ```
floor :: Impl.floor;
// The square root of `x` as a `float` or `float64`, correctly rounded. A negative argument gives NaN; `sqrt(-0.0)` is -0.0 and `sqrt(infinity)` is infinity.
//
// ```jai
// assert(sqrt(9.0) == 3 && sqrt(cast(float64) 2) > 1.4142 && sqrt(0.0) == 0);
// assert(isnan(sqrt(-1.0)));
// ```
sqrt :: Impl.sqrt;
// The sine of `x` in radians, as a `float` or `float64`. The result is in [-1, 1]; an infinite or NaN argument gives NaN. Accuracy for very large arguments is only as good as the platform's implementation.
//
// ```jai
// assert(sin(0.0) == 0);
// assert(abs(sin(PI / 2) - 1) < 0.00001);
// ```
sin :: Impl.sin;
// The cosine of `x` in radians, as a `float` or `float64`. The result is in [-1, 1]; an infinite or NaN argument gives NaN.
//
// ```jai
// assert(cos(0.0) == 1);
// assert(abs(cos(PI) + 1) < 0.00001);
// ```
cos :: Impl.cos;
// `sin(x)` when the compile-time flag `IS_SIN` is true, otherwise `cos(x)`, for `float64`. Lets generic code choose at compile time.
//
// ```jai
// assert(sin_or_cos(0.0, true) == 0 && sin_or_cos(0.0, false) == 1);
// ```
sin_or_cos :: Impl.sin_or_cos;
// Limits the value `pointer` points to into [`low`, `high`], in place. Unlike a `clamp` that returns a value, this edits the variable. With `low > high` the value ends at `high`. A NaN is left as it is, because every comparison with it is false.
//
// ```jai
// x := 15;
// Clamp(*x, 0, 10);
// assert(x == 10);
// ```
Clamp :: Impl.Clamp;
// The absolute value. For `s8`, `s16`, `s32` and `s64` it asserts on the most negative value, which has no positive counterpart. For `float` and `float64` it clears the sign bit, so `abs(-0.0)` is +0.0, `abs(-infinity)` is infinity and a NaN keeps its payload. Vectors take the absolute value of each component.
//
// ```jai
// assert(abs(-5) == 5 && abs(7) == 7);
// assert(abs(-2.5) == 2.5);
// assert(abs(xyz(-1, 2, -3)) == xyz(1, 2, 3));
// ```
abs :: Impl.abs;
// Returns two booleans for a `float` or `float64`: whether it is NaN, and whether it is an infinity of either sign.
//
// ```jai
// nan, inf := is_nan_is_inf(FLOAT32_INFINITY);
// assert(!nan && inf);
// nan, inf = is_nan_is_inf(FLOAT32_NAN);
// assert(nan && !inf);
// ```
is_nan_is_inf :: Impl.is_nan_is_inf;
// True when `x` is NaN. This is the only reliable test, because NaN compares unequal to itself.
//
// ```jai
// assert(isnan(FLOAT32_NAN) && !isnan(1.0) && !isnan(FLOAT32_INFINITY));
// ```
isnan :: Impl.isnan;
// True when `x` is positive or negative infinity. NaN is not infinite.
//
// ```jai
// assert(isinf(FLOAT32_INFINITY) && isinf(-FLOAT32_INFINITY));
// assert(!isinf(FLOAT32_MAX) && !isinf(FLOAT32_NAN));
// ```
isinf :: Impl.isinf;
// True when `x` is neither NaN nor infinite.
//
// ```jai
// assert(is_finite(1.0) && is_finite(FLOAT32_MAX));
// assert(!is_finite(FLOAT32_INFINITY) && !is_finite(FLOAT32_NAN));
// ```
is_finite :: Impl.is_finite;
// The fractional part `x - floor(x)`, always in [0, 1) for finite `x`. For a negative `x` this is not the part after the decimal point: `frac(-1.25)` is 0.75. Only `float` and `float64`.
//
// ```jai
// assert(frac(2.25) == 0.25 && frac(-1.25) == 0.75 && frac(3.0) == 0);
// ```
frac :: Impl.frac;
// `x` wrapped into [0, `divisor`) as `x - floor(x / divisor) * divisor`: the result has the sign of `divisor`, so a negative `x` cycles around instead of going negative (unlike the C `fmod`). A zero divisor prints a message to standard error and returns infinity.
//
// ```jai
// assert(fmod_cycling(7.0, 3.0) == 1);
// assert(fmod_cycling(-1.0, 3.0) == 2);
// assert(fmod_cycling(5.5, 2.0) == 1.5);
// ```
fmod_cycling :: Impl.fmod_cycling;
// Clamps to [0, 1]: `float`, `float64`, and each component of a vector. A NaN is returned as it is.
//
// ```jai
// assert(saturate(1.5) == 1 && saturate(-0.5) == 0 && saturate(0.25) == 0.25);
// assert(saturate(xyz(2, -1, 0.5)) == xyz(1, 0, 0.5));
// ```
saturate :: Impl.saturate;
// `a * a`, for any type with `*`.
//
// ```jai
// assert(square(3) == 9 && square(1.5) == 2.25);
// ```
square :: Impl.square;
// Rounds `x` down to a multiple of 1/16: `floor(16 * x) / 16`. Only `float64`.
//
// ```jai
// assert(reduc(cast(float64) 1.99) == 1.9375 && reduc(cast(float64) -0.01) == -0.0625);
// ```
reduc :: Impl.reduc;
// The linear interpolation `a + (b - a) * t`: `a` at `t = 0`, `b` at `t = 1`. `t` is not clamped, so values outside [0, 1] extrapolate. Works on `float`, `float64`, vectors, `Matrix3` (element by element) and quaternions (a plain component interpolation that is not normalized; see `nlerp` and `slerp`).
//
// ```jai
// assert(lerp(10.0, 20.0, 0.25) == 12.5);
// assert(lerp(10.0, 20.0, 2.0) == 30);
// assert(lerp(xyz(0, 0, 0), xyz(2, 4, 6), 0.5) == xyz(1, 2, 3));
// ```
lerp :: Impl.lerp;
// 1 when the sign bit of `x` is set and 0 otherwise, as a `u64`. Unlike `x < 0` it tells `-0.0` from `0.0` and sees the sign of a NaN. Only `float64`.
//
// ```jai
// assert(signbit(cast(float64) -1) == 1 && signbit(cast(float64) 1) == 0);
// assert(signbit(cast(float64) -0.0) == 1);
// ```
signbit :: Impl.signbit;
// Splits a `float64` into a fraction and a power of two, `x = fraction * 2^exponent` with `abs(fraction)` in [0.5, 1). Zero, infinities and NaN come back unchanged with exponent 0. Subnormals are handled.
//
// ```jai
// m, e := frexp(cast(float64) 8);
// assert(m == 0.5 && e == 4);
// m, e = frexp(cast(float64) -0.75);
// assert(m == -0.75 && e == 0);
// ```
frexp :: Impl.frexp;
// `x * 2^pw2`, computed exactly by exponent arithmetic (the inverse of `frexp`). An exponent beyond the range gives infinity or zero (with the sign of `x`), and the final underflow rounds once. Zero, infinities and NaN are returned unchanged. Only `float64`.
//
// ```jai
// assert(ldexp(cast(float64) 0.5, 4) == 8);
// assert(ldexp(cast(float64) 3, -1) == 1.5);
// assert(isinf(ldexp(cast(float64) 1, 2000)));
// ```
ldexp :: Impl.ldexp;
// The natural logarithm. `log(0)` is negative infinity, a negative or NaN argument gives NaN, and `log(infinity)` is infinity. `float` and `float64`; the `float` form computes in `float64` and rounds. Written in Jai, so the last digit can differ from a C library's `log`.
//
// ```jai
// assert(abs(log(cast(float64) 1)) < 0.0000001);
// assert(abs(log(cast(float64) 2.718281828459045) - 1) < 0.0000001);
// assert(log(cast(float64) 0) == -FLOAT64_INFINITY && isnan(log(cast(float64) -1)));
// ```
log :: Impl.log;
// `e` raised to `x`. Overflow gives infinity (above about 709.78 for `float64`), underflow gives zero, NaN stays NaN. `float` and `float64`; the `float` form computes in `float64` and rounds. Written in Jai, so the last digit can differ from a C library's `exp`.
//
// ```jai
// assert(exp(cast(float64) 0) == 1);
// assert(abs(exp(cast(float64) 1) - 2.718281828459045) < 0.000001);
// assert(isinf(exp(cast(float64) 1000)) && exp(cast(float64) -1000) == 0);
// ```
exp :: Impl.exp;
// `x` raised to `y`, for `float` and `float64` (the `float` form computes in `float64` and rounds). Whole exponents below 2^31 use repeated multiplication, so `pow(x, 2)` is exactly `x * x`. Edge cases: `y == 0` or `x == 1` gives 1 even for NaN; any other NaN gives NaN; a negative `x` with a non-whole `y` gives NaN; a zero `x` with negative `y` gives infinity.
//
// ```jai
// assert(pow(cast(float64) 2, 10) == 1024);
// assert(pow(cast(float64) 2, -2) == 0.25);
// assert(pow(cast(float64) -2, 3) == -8);
// assert(isnan(pow(cast(float64) -8, 0.5)));
// assert(pow(FLOAT64_NAN, 0) == 1);
// ```
pow :: Impl.pow;
// The arc tangent of `x` in radians, in (-pi/2, pi/2). `float` and `float64`. NaN and zero are returned as they are.
//
// ```jai
// assert(atan(cast(float64) 0) == 0);
// assert(abs(atan(cast(float64) 1) - PI64 / 4) < 0.000001);
// ```
atan :: Impl.atan;
// The angle of the point (`x`, `y`) from the positive x axis in radians, in [-pi, pi], using the signs of both to pick the quadrant. Note the argument order: `y` first. `atan2(0, x)` is 0 for positive `x` and pi for negative `x` (negative pi when `y` is -0.0); `atan2(0, 0)` is 0. NaN in either argument gives NaN.
//
// ```jai
// assert(abs(atan2(cast(float64) 1, cast(float64) 1) - PI64 / 4) < 0.000001);
// assert(abs(atan2(cast(float64) 1, cast(float64) -1) - 3 * PI64 / 4) < 0.000001);
// assert(abs(atan2(cast(float64) -1, cast(float64) 0) + PI64 / 2) < 0.000001);
// ```
atan2 :: Impl.atan2;
// The arc sine in radians, in [-pi/2, pi/2]. `float` and `float64`. An argument outside [-1, 1] gives NaN.
//
// ```jai
// assert(abs(asin(cast(float64) 1) - PI64 / 2) < 0.000001);
// assert(isnan(asin(cast(float64) 1.5)));
// ```
asin :: Impl.asin;
// The arc cosine in radians, in [0, pi]. `float` and `float64`. An argument outside [-1, 1] gives NaN.
//
// ```jai
// assert(acos(cast(float64) 1) == 0);
// assert(abs(acos(cast(float64) -1) - PI64) < 0.000001);
// assert(isnan(acos(cast(float64) 2)));
// ```
acos :: Impl.acos;
// The tangent of `x` in radians, computed as `sin(x) / cos(x)`, so it is huge but finite near odd multiples of pi/2 and gives NaN for an infinite argument. `float` and `float64`.
//
// ```jai
// assert(tan(0.0) == 0);
// assert(abs(tan(PI / 4) - 1) < 0.0001);
// ```
tan :: Impl.tan;
// The cotangent `cos(x) / sin(x)` for `float64`. At a multiple of pi the sine is (nearly) zero, so the result is huge or infinite.
//
// ```jai
// assert(abs(cot(PI64 / 4) - 1) < 0.000001);
// ```
cot :: Impl.cot;
// Converts the 16 bits of an IEEE half-precision float to a `float`, exactly (subnormals, infinities and NaN included). The `Float16` module wraps this with typed halves and the opposite direction.
//
// ```jai
// assert(half_to_float(0x3c00) == 1.0);
// assert(half_to_float(0xc000) == -2.0);
// assert(half_to_float(0x7bff) == 65504.0);
// ```
half_to_float :: Impl.half_to_float;
// The dot product of two vectors (or two quaternions), the sum of the products of matching components. For unit vectors it is the cosine of the angle between them. Also spelled `dot`.
//
// ```jai
// assert(dot_product(xyz(1, 2, 3), xyz(4, 5, 6)) == 32);
// assert(dot(xy(1, 0), xy(0, 1)) == 0);
// ```
dot_product :: Impl.dot_product;
// The squared length of a vector, `dot(a, a)`. Cheaper than `length` for comparisons; it can overflow to infinity where `length` does not.
//
// ```jai
// assert(length_squared(xyz(1, 2, 2)) == 9);
// ```
length_squared :: Impl.length_squared;
// The Euclidean length of a vector or quaternion. The sum of squares is scaled by the largest component first, so a vector of huge or tiny components does not overflow or underflow in the middle: `length(xy(FLOAT32_MAX, FLOAT32_MAX))` is about 4.8e38, which does not fit a `float`, so it is infinity. Any NaN component gives NaN, and an infinite one infinity (unless there is a NaN as well).
//
// ```jai
// assert(length(xy(3, 4)) == 5);
// assert(length(xyzw(1, 1, 1, 1)) == 2);
// assert(length(xyz(0, 0, 0)) == 0);
// assert(isinf(length(xy(FLOAT32_INFINITY, 1))));
// ```
length :: Impl.length;
// The component-wise minimum of two vectors. For scalars use the `min` of the `Basic` module.
//
// ```jai
// assert(min(xyz(1, 5, 3), xyz(4, 2, 6)) == xyz(1, 2, 3));
// ```
min :: Impl.min;
// The component-wise maximum of two vectors. For scalars use the `max` of the `Basic` module.
//
// ```jai
// assert(max(xyz(1, 5, 3), xyz(4, 2, 6)) == xyz(4, 5, 6));
// ```
max :: Impl.max;
// A `Vector2` with both components set to `x`. To build one from two numbers, write `xy(x, y)` or `Vector2.{ x, y }`.
//
// ```jai
// assert(make_vector2(2) == xy(2, 2));
// ```
make_vector2 :: Impl.make_vector2;
// Builds a `Vector2` from two components, or from one number for both.
//
// ```jai
// assert(xy(1, 2) == Vector2.{ 1, 2 } && xy(3) == xy(3, 3));
// ```
xy :: Impl.xy;
// Multiplies a vector by a scalar (`v * f`), or applies a matrix to a column vector (`m * v`) or to another matrix (`a * b`). The operator `*` is the same for every pairing here: this is the named form of it, convenient to pass as a procedure. See each matrix type for the shapes.
//
// ```jai
// assert(multiply(xyz(1, 2, 3), 2) == xyz(2, 4, 6));
// assert(multiply(Matrix3_Identity, xyz(1, 2, 3)) == xyz(1, 2, 3));
// ```
multiply :: Impl.multiply;
// The squared distance between two points, `length_squared(a - b)`.
//
// ```jai
// assert(distance_squared(xy(1, 1), xy(4, 5)) == 25);
// ```
distance_squared :: Impl.distance_squared;
// `v` scaled to length 1, or `v` unchanged when it is zero (a zero vector has no direction, so it is returned as it is, signs of zero kept). Works at full precision for very small and very large components, which a plain divide by `length` would overflow or flush to zero. To also get the old length, or to supply a fallback, use `normalize`.
//
// ```jai
// assert(unit_vector(xyz(0, 0, 5)) == xyz(0, 0, 1));
// assert(unit_vector(xyz(0, 0, 0)) == xyz(0, 0, 0));
// n := unit_vector(xy(3, 4));
// assert(abs(n.x - 0.6) < 0.00001 && abs(n.y - 0.8) < 0.00001);
// ```
unit_vector :: Impl.unit_vector;
// The distance between two points, `length(a - b)`.
//
// ```jai
// assert(distance(xyz(1, 1, 1), xyz(1, 4, 5)) == 5);
// ```
distance :: Impl.distance;
// A `Vector3` with all components set to `x`, or one built from an array of three floats, or from a `Vector2` and a `z`. To build one from three numbers, write `xyz(x, y, z)` or `Vector3.{ x, y, z }`.
//
// ```jai
// assert(make_vector3(2) == xyz(2, 2, 2));
// assert(make_vector3(xy(1, 2), 3) == xyz(1, 2, 3));
// ```
make_vector3 :: Impl.make_vector3;
// Builds a `Vector3` from three components, from one number for all three, from an array of three floats, or from a `Vector2` and a `z`.
//
// ```jai
// assert(xyz(1, 2, 3) == Vector3.{ 1, 2, 3 } && xyz(4) == xyz(4, 4, 4));
// assert(xyz(xy(1, 2), 3) == xyz(1, 2, 3));
// ```
xyz :: Impl.xyz;
// A `Vector4` with all components set to `x`, or one built from a `Vector3` and a `w`, or a `Vector2` and a `z` and `w`. To build one from four numbers, write `xyzw(x, y, z, w)` or `Vector4.{ x, y, z, w }`.
//
// ```jai
// assert(make_vector4(1) == xyzw(1, 1, 1, 1));
// assert(make_vector4(xyz(1, 2, 3), 1) == xyzw(1, 2, 3, 1));
// ```
make_vector4 :: Impl.make_vector4;
// Builds a `Vector4` from four components, from one number for all four, from a `Vector3` and a `w`, or from a `Vector2` plus `z` and `w`.
//
// ```jai
// assert(xyzw(1, 2, 3, 4) == Vector4.{ 1, 2, 3, 4 });
// assert(xyzw(xy(1, 2), 3, 4) == xyzw(1, 2, 3, 4));
// ```
xyzw :: Impl.xyzw;
// The same procedure as `dot_product`.
dot :: dot_product;
// The same procedure as `xyz`, named for colors: red, green and blue as a `Vector3`.
rgb :: Impl.rgb;
// The same procedure as `xyzw`, named for colors: red, green, blue and alpha as a `Vector4`.
rgba :: Impl.rgba;
// Scales a vector or quaternion to length 1. Given a pointer it normalizes in place and returns the length it had; given a value it returns the normalized copy.
//
// If the length is not above `epsilon` (default 0, so only an exactly zero length), the vector is not scaled: it is replaced by `fallback` (default all zeros). The comparison is against the rounded `float` length. Components are scaled at `float64` precision, so tiny and huge vectors keep their direction. A `Plane3` is normalized by its normal's length, so the normal is unit length and the plane stays the same plane.
//
// ```jai
// v := xyz(0, 3, 4);
// n := normalize(*v);
// assert(n == 5 && v == xyz(0, 0.6, 0.8));
// assert(normalize(xy(0, 2)) == xy(0, 1));
// w := xy(0, 0);
// normalize(*w, fallback = xy(1, 0));
// assert(w == xy(1, 0));
// ```
normalize :: Impl.normalize;
// The same procedure as `cross_product`.
cross :: Impl.cross;
// The cross product of two `Vector3`: perpendicular to both, with length `|a||b|sin(angle)`, and zero when they are parallel. Right-handed: `x` cross `y` is `z`. Also spelled `cross`.
//
// ```jai
// assert(cross(xyz(1, 0, 0), xyz(0, 1, 0)) == xyz(0, 0, 1));
// assert(cross(xyz(0, 1, 0), xyz(1, 0, 0)) == xyz(0, 0, -1));
// ```
cross_product :: Impl.cross_product;
// Rotates a vector or a matrix.
//
// - `rotate(v: Vector2, theta)` turns the vector counter-clockwise by `theta` radians; a pointer rotates in place.
// - `rotate(v: Vector3, q)` applies a unit quaternion `q`; a pointer rotates in place.
// - `rotate(m: Matrix3 or Matrix4, q)` returns `m * rotation_matrix(q)`, so the rotation is applied before `m`'s own transform.
//
// ```jai
// v := rotate(xy(1, 0), PI / 2);
// assert(abs(v.x) < 0.0001 && abs(v.y - 1) < 0.0001);
//
// q: Quaternion;
// set_from_axis_and_angle(*q, xyz(0, 0, 1), PI);
// w := rotate(xyz(1, 0, 0), q);
// assert(abs(w.x + 1) < 0.0001 && abs(w.y) < 0.0001);
// ```
rotate :: Impl.rotate;
// Normalizes a quaternion in place and returns the length it had. A quaternion of length zero is set to the identity `Quaternion.{}` instead of dividing by zero.
//
// ```jai
// q := Quaternion.{ 0, 0, 0, 2 };
// assert(normalize_or_identity(*q) == 2 && q == Quaternion.{ 0, 0, 0, 1 });
// z := Quaternion.{ 0, 0, 0, 0 };
// normalize_or_identity(*z);
// assert(z == Quaternion.{});
// ```
normalize_or_identity :: Impl.normalize_or_identity;
// Interpolates between two quaternions by blending their components and normalizing the result: cheap, with a non-constant angular speed. It does not take the shorter way around: if the two quaternions are more than 90 degrees apart (negative `dot_product`), negate one first, as `slerp` does.
//
// ```jai
// a := Quaternion.{};
// b: Quaternion;
// set_from_axis_and_angle(*b, xyz(0, 0, 1), PI / 2);
// m := nlerp(a, b, 0.5);
// assert(abs(length(m) - 1) < 0.0001);
// assert(nlerp(a, b, 0) == a);
// ```
nlerp :: Impl.nlerp;
// Spherical interpolation between two unit quaternions: constant angular speed from `start` at `t = 0` to `end` at `t = 1`. If the two are on opposite sides (negative dot product) it interpolates towards the negated `end`, so it takes the shorter rotation. Very close quaternions use `nlerp` to avoid dividing by a near-zero sine. The inputs must be unit length; `t` outside [0, 1] extrapolates.
//
// ```jai
// a := Quaternion.{};
// b: Quaternion;
// set_from_axis_and_angle(*b, xyz(0, 0, 1), PI / 2);
// half := slerp(a, b, 0.5);
// _, _, _, angle := get_axis_and_angle(half);
// assert(abs(angle - PI64 / 4) < 0.0001);
// ```
slerp :: Impl.slerp;
// Sets a quaternion to the rotation by `theta` radians about `v` (or `x, y, z`), counter-clockwise looking down the axis. The axis must be unit length; it is not normalized here.
//
// ```jai
// q: Quaternion;
// set_from_axis_and_angle(*q, xyz(0, 0, 1), PI);
// assert(abs(q.z - 1) < 0.0001 && abs(q.w) < 0.0001);
// set_from_axis_and_angle(*q, 1, 0, 0, 0);
// assert(q == Quaternion.{});
// ```
set_from_axis_and_angle :: Impl.set_from_axis_and_angle;
// Returns the rotation of a quaternion as `x, y, z, theta`: a unit axis and the angle in radians, in `float64`. The identity (no rotation) gives the axis `(1, 0, 0)` and angle 0. The quaternion should be unit length; for a quaternion with negative `w` the angle is above pi (the rotation the other way round).
//
// ```jai
// q: Quaternion;
// set_from_axis_and_angle(*q, xyz(0, 1, 0), 1.0);
// x, y, z, theta := get_axis_and_angle(q);
// assert(abs(y - 1) < 0.0001 && abs(x) < 0.0001 && abs(z) < 0.0001 && abs(theta - 1) < 0.0001);
// ```
get_axis_and_angle :: Impl.get_axis_and_angle;
// The conjugate `(-x, -y, -z, w)`. For a unit quaternion this is the inverse rotation.
//
// ```jai
// q: Quaternion;
// set_from_axis_and_angle(*q, xyz(0, 0, 1), 1.0);
// p := q * conjugate(q);
// assert(abs(p.w - 1) < 0.0001 && abs(p.z) < 0.0001);
// ```
conjugate :: Impl.conjugate;
// Writes the 3x3 rotation matrix of a unit quaternion into a `Matrix3` or any matrix with the 3x3 fields (a `Matrix4` keeps the rest of it unchanged). The matrix then rotates column vectors: `m * v` equals `rotate(v, q)`.
//
// ```jai
// q: Quaternion;
// set_from_axis_and_angle(*q, xyz(0, 0, 1), PI / 2);
// m: Matrix3;
// set_rotation(*m, q);
// v := m * xyz(1, 0, 0);
// assert(abs(v.x) < 0.0001 && abs(v.y - 1) < 0.0001);
// ```
set_rotation :: Impl.set_rotation;
// The rotation matrix of a unit quaternion as a new matrix of type `T` (`Matrix3` or `Matrix4`, which has an identity translation).
//
// ```jai
// q: Quaternion;
// set_from_axis_and_angle(*q, xyz(0, 0, 1), PI / 2);
// m := rotation_matrix(Matrix4, q);
// assert(m._44 == 1 && m._14 == 0);
// assert(abs((m * xyzw(1, 0, 0, 1)).y - 1) < 0.0001);
// ```
rotation_matrix :: Impl.rotation_matrix;
// Extracts a unit quaternion from the 3x3 part of a rotation matrix. The matrix must be a pure rotation (orthonormal, determinant 1); a scale or a reflection gives a meaningless result. The sign of the quaternion is not determined (`q` and `-q` are the same rotation).
//
// ```jai
// q: Quaternion;
// set_from_axis_and_angle(*q, xyz(1, 0, 0), 0.5);
// back := get_rotation(rotation_matrix(Matrix3, q));
// assert(abs(dot_product(q, back)) > 0.9999);
// ```
get_rotation :: Impl.get_rotation;
// Returns two unit vectors `(y_axis, z_axis)` that together with `x_axis` (which need not be unit length) form a right-handed orthonormal basis, `y = z cross x`. With one argument a helper direction is picked automatically; with `approximate_axis` the `z` axis is made perpendicular to `x_axis` and that direction. If the helper is parallel to `x_axis` another one is substituted.
//
// ```jai
// y, z := make_an_orthonormal_basis(xyz(1, 0, 0));
// assert(abs(dot(y, z)) < 0.0001 && abs(dot(y, xyz(1, 0, 0))) < 0.0001);
// assert(abs(length(y) - 1) < 0.0001 && abs(length(z) - 1) < 0.0001);
// ```
make_an_orthonormal_basis :: Impl.make_an_orthonormal_basis;
// Row `i` (0-based) of a `Matrix4` as a `Vector4`. No range check.
//
// ```jai
// m := Matrix4_Identity;
// m._14 = 5;
// assert(row(m, 0) == xyzw(1, 0, 0, 5));
// ```
row :: Impl.row;
// Column `i` (0-based) of a `Matrix4` as a `Vector4`. Column 3 is the translation. No range check.
//
// ```jai
// m := make_translation_matrix4(xyz(1, 2, 3));
// assert(column(m, 3) == xyzw(1, 2, 3, 1));
// ```
column :: Impl.column;
// Transforms a direction by the 3x3 part of a matrix (`Matrix3`, `Matrix4`, `Matrix4x3`): rotation and scale apply, translation does not. Use `transform_point` for positions.
//
// ```jai
// m := make_translation_matrix4(xyz(10, 0, 0));
// assert(transform_vector(m, xyz(1, 2, 3)) == xyz(1, 2, 3));
// ```
transform_vector :: Impl.transform_vector;
// Transforms a position by the 3x3 part of a `Matrix4` or `Matrix4x3` and then adds its translation column, treating the point as having `w = 1`. There is no perspective divide, so for a projection matrix multiply a `Vector4` and divide by `w` yourself.
//
// ```jai
// m := make_translation_matrix4(xyz(10, 0, 0));
// assert(transform_point(m, xyz(1, 2, 3)) == xyz(11, 2, 3));
// ```
transform_point :: Impl.transform_point;
// The upper-left 3x3 of any matrix with those fields, as a `Matrix3`.
//
// ```jai
// m := make_translation_matrix4(xyz(1, 2, 3));
// assert(make_matrix3(m) == Matrix3_Identity);
// ```
make_matrix3 :: Impl.make_matrix3;
// Extends a 3x3 matrix to a `Matrix4`: the 3x3 in the corner, zero translation, and 1 at `_44`.
//
// ```jai
// m := make_matrix4(Matrix3_Identity);
// assert(m == Matrix4_Identity);
// ```
make_matrix4 :: Impl.make_matrix4;
// The transpose of a `Matrix3` or `Matrix4` (rows become columns), as a new matrix; given a pointer, it transposes in place. For a rotation matrix the transpose is the inverse.
//
// ```jai
// m := Matrix3.{ 1, 2, 3, 4, 5, 6, 7, 8, 9 };
// t := transpose(m);
// assert(t._12 == 4 && t._21 == 2);
// transpose(*m);
// assert(m == t);
// ```
transpose :: Impl.transpose;
// A new identity matrix of the matrix type `T`.
//
// ```jai
// assert(identity_of(Matrix4) == Matrix4_Identity);
// ```
identity_of :: Impl.identity_of;
// Sets the matrix `m` points to the identity, for any of the matrix types. `Matrix4x3` and `Matrix4` get zero translation.
//
// ```jai
// m: Matrix3;
// m._12 = 4;
// identity(*m);
// assert(m == Matrix3_Identity);
// ```
identity :: Impl.identity;
// A `Matrix4` that scales x, y and z by the components of `v` and leaves the translation at zero.
//
// ```jai
// m := make_scale_matrix4(xyz(2, 3, 4));
// assert(transform_point(m, xyz(1, 1, 1)) == xyz(2, 3, 4));
// ```
make_scale_matrix4 :: Impl.make_scale_matrix4;
// Multiplies the columns of a matrix by `s` (all three) or by `v.x`, `v.y`, `v.z` respectively, which is `m * scale_matrix`: the scale is applied to points first, then `m`'s own transform. Works on `Matrix3` and `Matrix4`; a pointer scales in place. The translation column of a `Matrix4` is not scaled.
//
// ```jai
// m := Matrix3_Identity;
// scaled := scale(m, 2.0);
// assert(scaled * xyz(1, 1, 1) == xyz(2, 2, 2));
// m2 := scale(Matrix3_Identity, xyz(1, 2, 3));
// assert(m2 * xyz(1, 1, 1) == xyz(1, 2, 3));
// ```
scale :: Impl.scale;
// A `Matrix4` that moves points by `v`: the identity with `v` in the last column.
//
// ```jai
// m := make_translation_matrix4(xyz(1, 2, 3));
// assert(m._14 == 1 && m._24 == 2 && m._34 == 3);
// assert(m * xyzw(0, 0, 0, 1) == xyzw(1, 2, 3, 1));
// ```
make_translation_matrix4 :: Impl.make_translation_matrix4;
// Adds `t` to the translation column of a `Matrix4` and returns the result; given a pointer it changes the matrix. The move is in the output space (after the matrix's own transform), so it equals `make_translation_matrix4(t) * m`.
//
// ```jai
// m := make_scale_matrix4(xyz(2, 2, 2));
// moved := translate(m, xyz(5, 0, 0));
// assert(transform_point(moved, xyz(1, 1, 1)) == xyz(7, 2, 2));
// ```
translate :: Impl.translate;
// A matrix whose first three rows are the vectors `xprime`, `yprime`, `zprime` (the rest as in the identity). The type defaults to `Matrix4`; pass `Matrix3` as the last argument for that. Multiplying a point by it takes the point's components along those three directions.
//
// ```jai
// m := make_matrix_from_rows(xyz(0, 1, 0), xyz(1, 0, 0), xyz(0, 0, 1));
// assert(m * xyzw(1, 2, 3, 1) == xyzw(2, 1, 3, 1));
// ```
make_matrix_from_rows :: Impl.make_matrix_from_rows;
// A matrix whose first three columns are `xprime`, `yprime`, `zprime`: it sends the x, y and z axes to those vectors. The type defaults to `Matrix4`; pass `Matrix3` as the last argument for that.
//
// ```jai
// m := make_matrix_from_columns(xyz(0, 1, 0), xyz(1, 0, 0), xyz(0, 0, 1));
// assert(m * xyzw(1, 0, 0, 1) == xyzw(0, 1, 0, 1));
// ```
make_matrix_from_columns :: Impl.make_matrix_from_columns;
// A view matrix for a camera at `viewpoint` looking at `look_at`, with `reference_up_vector` fixing the roll. It maps world positions into camera space, where the camera sits at the origin. `look_at` must not equal `viewpoint`, and the up vector must not be parallel to the view direction (the result is NaN or garbage).
//
// `x_is_forward` (default true) picks the camera axes. With it the camera looks along +x, with +z up and -y to the right. With `x_is_forward = false` it looks along -z, with +y up and +x to the right: the usual OpenGL view space, which is what `make_projection_matrix` and `make_frustum_matrix` expect.
//
// ```jai
// view := make_look_at_matrix(xyz(0, 0, 5), xyz(0, 0, 0), xyz(0, 1, 0), x_is_forward = false);
// p := transform_point(view, xyz(0, 0, 0));
// assert(abs(p.x) < 0.0001 && abs(p.y) < 0.0001 && abs(p.z + 5) < 0.0001); // 5 units in front (-z)
// ```
make_look_at_matrix :: Impl.make_look_at_matrix;
// An orthographic projection of the box `left..right`, `bottom..top`, `near..far` (distances in front of the camera, which looks along -z) into clip space. The depth range maps to -1..1, or 0..1 with `depth_range_01 = true`. Right-handed; no divide by zero check, so equal bounds give infinities.
//
// ```jai
// m := orthographic_projection_matrix(-2, 2, -1, 1, 1, 11);
// p := m * xyzw(2, 1, -1, 1); // right, top, near
// assert(abs(p.x - 1) < 0.0001 && abs(p.y - 1) < 0.0001 && abs(p.z + 1) < 0.0001);
// ```
orthographic_projection_matrix :: Impl.orthographic_projection_matrix;
// A perspective projection from the frustum with its near plane at distance `n` spanning `l..r` by `b..t`, and its far plane at `f`. The camera looks along -z, clip depth is -1..1 (0..1 with `depth_range_01 = true`), and after multiplying a `Vector4` you divide by `w` to get normalized coordinates. `n` and `f` must be positive and `n != f`.
//
// ```jai
// m := make_frustum_matrix(-1, 1, -1, 1, 1, 10);
// p := m * xyzw(0, 0, -1, 1); // on the near plane
// assert(abs(p.z / p.w + 1) < 0.0001);
// p = m * xyzw(0, 0, -10, 1); // on the far plane
// assert(abs(p.z / p.w - 1) < 0.0001);
// ```
make_frustum_matrix :: Impl.make_frustum_matrix;
// A symmetric perspective projection from a vertical field of view in radians, the aspect ratio (width over height), and the near and far distances. Right-handed, camera looks along -z, clip depth -1..1 (0..1 with `depth_range_01 = true`). `x_offset` and `y_offset` shear the frustum for jittering or off-center views. The field of view must be in (0, pi) and `z_near` and `z_far` positive and different.
//
// ```jai
// m := make_projection_matrix(PI / 2, 1.0, 1, 100);
// p := m * xyzw(0, 0, -1, 1);
// assert(abs(p.x) < 0.0001 && abs(p.w - 1) < 0.0001);
// p = m * xyzw(1, 1, -1, 1); // a point at 45 degrees lands at the edge of clip space
// assert(abs(p.x / p.w - 1) < 0.0001 && abs(p.y / p.w - 1) < 0.0001);
// ```
make_projection_matrix :: Impl.make_projection_matrix;
// The matrix of cofactors of a `Matrix3`: its rows are the cross products of the other two rows. Dividing its transpose by the determinant gives the inverse.
//
// ```jai
// m := Matrix3.{ 2, 0, 0, 0, 3, 0, 0, 0, 4 };
// assert(adjoint_transpose(m) == Matrix3.{ 12, 0, 0, 0, 8, 0, 0, 0, 6 });
// ```
adjoint_transpose :: Impl.adjoint_transpose;
// The inverse of a matrix, returned as `(success, result)`.
//
// - `Matrix3`: from the determinant. Fails when the determinant is within `epsilon` (default 0.001) of zero, which is an absolute limit, so a valid matrix with small entries can be reported as not invertible; lower `epsilon` for those. On failure `result` is the input.
// - `Matrix4`: by Gaussian elimination with row pivoting (`inverse_gaussian_elimination`). Fails only when a pivot is exactly zero. On failure `result` is the identity.
//
// ```jai
// ok, inv := inverse(Matrix3.{ 2, 0, 0, 0, 4, 0, 0, 0, 5 });
// assert(ok && inv._11 == 0.5 && inv._22 == 0.25 && inv._33 == 0.2);
//
// ok4, inv4 := inverse(Matrix4.{});
// assert(!ok4 && inv4 == Matrix4_Identity);
// ```
inverse :: Impl.inverse;
// Inverts a `Matrix4` by Gauss-Jordan elimination with partial pivoting, returning `(success, result)`. Fails only when a pivot is exactly zero (singular), giving the identity as `result`. Nearly singular matrices succeed with a large error.
//
// ```jai
// t := make_translation_matrix4(xyz(1, 2, 3));
// ok, inv := inverse_gaussian_elimination(t);
// assert(ok && inv * t == Matrix4_Identity);
// ```
inverse_gaussian_elimination :: Impl.inverse_gaussian_elimination;
// The adjugate (transposed cofactor matrix) of a `Matrix4`, so that `adjugate(m) * m` is the determinant times the identity. Also named `adjugate`.
//
// ```jai
// m := make_scale_matrix4(xyz(2, 3, 4));
// a := adjugate(m);
// assert(a * m == Matrix4_Identity * determinant(m));
// ```
adjugate_lengyel :: Impl.adjugate_lengyel;
// The same procedure as `adjugate_lengyel`.
adjugate :: Impl.adjugate;
// The determinant of a `Matrix4`, by cofactor expansion along the first row, as a `float`. Zero means the matrix is not invertible; the sign tells whether it flips handedness.
//
// ```jai
// assert(determinant(Matrix4_Identity) == 1);
// assert(determinant(make_scale_matrix4(xyz(2, 3, 4))) == 24);
// assert(determinant(make_scale_matrix4(xyz(-1, 1, 1))) == -1);
// ```
determinant :: Impl.determinant;
// Inverts a `Matrix4` through its adjugate and determinant, returning `(success, result)`. Fails when `abs(determinant)` is below `epsilon` (default 0.0001, an absolute limit), giving the identity as `result`. Less robust than `inverse` for badly scaled matrices.
//
// ```jai
// ok, inv := inverse_lengyel(make_scale_matrix4(xyz(2, 4, 8)));
// assert(ok && inv._11 == 0.5 && inv._22 == 0.25 && inv._33 == 0.125);
// ```
inverse_lengyel :: Impl.inverse_lengyel;
// The inverse of a `Matrix4` that is a rotation plus a translation only: transposes the 3x3 part and rotates the negated translation. Much cheaper than a general inverse, but wrong for any matrix with scale, shear or projection.
//
// ```jai
// q: Quaternion;
// set_from_axis_and_angle(*q, xyz(0, 0, 1), 0.7);
// m := rotation_matrix(Matrix4, q);
// m._14 = 3;
// inv := isometry_inverse(m);
// p := transform_point(inv, transform_point(m, xyz(1, 2, 3)));
// assert(abs(p.x - 1) < 0.0001 && abs(p.y - 2) < 0.0001 && abs(p.z - 3) < 0.0001);
// ```
isometry_inverse :: Impl.isometry_inverse;
// The maximum absolute row sum of a `Matrix3` (the infinity norm), or with `tpose` true the maximum absolute column sum (the one norm). `norm_inf` and `norm_one` are the two fixed forms.
//
// ```jai
// m := Matrix3.{ 1, -2, 0, 0, 1, 1, 4, 0, 0 };
// assert(mat_norm(m, false) == 4 && mat_norm(m, true) == 5);
// ```
mat_norm :: Impl.mat_norm;
// The maximum absolute row sum of a `Matrix3`: `mat_norm(m, false)`.
norm_inf :: Impl.norm_inf;
// The maximum absolute column sum of a `Matrix3`: `mat_norm(m, true)`.
norm_one :: Impl.norm_one;
// Returns the vector `u` of the Householder reflection `I - u u^T` that maps `v` onto the z axis (to minus its own length along z when `v.z` is not negative). `u` has length square root of 2, which is what makes `I - u u^T` a reflection; `reflect_cols` and `reflect_rows` apply it. A zero `v` gives the zero vector.
//
// ```jai
// v := xyz(1, 2, 2);
// h := Matrix3_Identity;
// reflect_cols(*h, make_reflector(v));
// r := h * v;
// assert(abs(r.x) < 0.0001 && abs(r.y) < 0.0001 && abs(r.z + 3) < 0.0001);
// ```
make_reflector :: Impl.make_reflector;
// Multiplies the matrix pointed to by the reflection `I - u u^T` from the right, in place: every row vector `v[i]` becomes `v[i] - u * dot(v[i], u)`. `u` should come from `make_reflector` (length square root of 2).
reflect_cols :: Impl.reflect_cols;
// Multiplies the matrix pointed to by the reflection `I - u u^T` from the left, in place: the same reflection as `reflect_cols` applied to the columns instead of the rows (it transposes, reflects and transposes back).
reflect_rows :: Impl.reflect_rows;
// Splits a `Matrix3` into `Q * S` (the polar decomposition), returned as `(Q, S, det)`: `Q` is an orthogonal matrix (a rotation, or a rotation with a flip when the determinant is negative), `S` is symmetric positive semi-definite (the stretch), and `det` is the determinant of the input. A singular input is allowed: the missing directions of `Q` are filled in so it stays orthogonal. Computed by an iterative eigen-solver at `float` precision, so `Q * S` matches the input to about 1e-6 for well-scaled input.
//
// ```jai
// m := Matrix3.{ 0, -2, 0, 3, 0, 0, 0, 0, 1 };
// q, s, det := polar_decomp(m);
// assert(abs(det - 6) < 0.0001);
// back := q * s;
// for 0..8 assert(abs(back.floats[it] - m.floats[it]) < 0.0001);
// ```
polar_decomp :: Impl.polar_decomp;
// Builds a `Plane3` from four coefficients `a, b, c, d`, a `Vector4`, a normal and a `d`, or a normal and a point on the plane (then `d = -dot(normal, point)`). The normal is stored as given; it is not normalized.
//
// ```jai
// p := make_plane(xyz(0, 0, 1), xyz(0, 0, 3)); // the plane z = 3
// assert(p.d == -3);
// assert(make_plane(1, 2, 3, 4).d == 4 && make_plane(xyz(1, 2, 3), 4).normal == xyz(1, 2, 3));
// ```
make_plane :: Impl.make_plane;
// A plane through `point` with the given `normal`: `make_plane(normal, point)`.
//
// ```jai
// p := plane_from_point_and_normal(xyz(0, 5, 0), xyz(0, 1, 0));
// assert(plane_dot(p, xyz(7, 8, 7)) == 3);
// ```
plane_from_point_and_normal :: Impl.plane_from_point_and_normal;
// The `(a, b, c)` normal of a plane (the same as its `normal` field).
get_normal :: Impl.get_normal;
// The plane through three points, with a unit normal `cross(p1 - p0, p2 - p0)`: the normal points towards the side from which `p0, p1, p2` appear counter-clockwise. Collinear points give a zero normal.
//
// ```jai
// p := normalized_plane_from_points(xyz(0, 0, 0), xyz(1, 0, 0), xyz(0, 1, 0));
// assert(p.normal == xyz(0, 0, 1) && p.d == 0);
// ```
normalized_plane_from_points :: Impl.normalized_plane_from_points;
// `dot(plane.normal, v) + plane.d`: the signed distance from point `v` to the plane if the normal has unit length (positive on the side of the normal, zero on the plane), otherwise that distance times the normal's length. The procedure is `#symmetric`, so the point may come first.
//
// ```jai
// p := make_plane(xyz(0, 1, 0), xyz(0, 2, 0));
// assert(plane_dot(p, xyz(0, 5, 0)) == 3 && plane_dot(p, xyz(0, 0, 0)) == -2);
// assert(plane_dot(xyz(0, 5, 0), p) == 3);
// ```
plane_dot :: Impl.plane_dot;
// A struct printer that prints a `Matrix2`, `Matrix3`, `Matrix4` or `Matrix4x3` as one row per line, each row as `(a, b, c)` with six decimal places and followed by a newline. Returns false for any other type. See `math_struct_printer` for how to install it.
print_matrix :: Impl.print_matrix;
// A struct printer for the types of this module: a vector or quaternion prints as `(x, y, z)`, a plane as its four coefficients and a matrix one row per line (via `print_matrix`), every number with six decimal places. It returns false for any other type, so the default formatting applies. Install it with `context.print_style.struct_printer = math_struct_printer;` to make `print("%", v)` use it.
//
// ```jai
// #import "Basic";
// context.print_style.struct_printer = math_struct_printer;
// text := tprint("%", xyz(1, 2, 3.5));
// assert(text == "(1.000000, 2.000000, 3.500000)");
// assert(tprint("%", Quaternion.{}) == "(0.000000, 0.000000, 0.000000, 1.000000)");
// ```
math_struct_printer :: Impl.math_struct_printer;

using,only(.["+", "-", "*", "/", "=="]) #import "internal/numerics/math";

#scope_file
Impl :: #import "internal/numerics/math";
