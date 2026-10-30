# Math

## Public declaration inventory

Visible public declarations and module-conditional variants are listed below. Procedure bodies and aggregate initializers are omitted; complete public type members are listed separately.

- `ceil :: (x: float64) -> float64`
- `floor :: (x: float64) -> float64`
- `ceil :: (f: float) -> float`
- `floor :: (f: float) -> float`
- `DP1 :: 7.85398125648498535156E-1`
- `DP2 :: 3.77489470793079817668E-8`
- `DP3 :: 2.69515142907905952645E-15`
- `LOSSTH : float64 : 1.073741824e9`
- `sin :: (x: float32) -> float32`
- `cos :: (x: float32) -> float32`
- `sin_or_cos :: (x: float64, $IS_SIN: bool) -> float64`
- `sin :: (x: float64) -> float64`
- `cos :: (x: float64) -> float64`
- `atan :: inline (x: float64) -> float64`
- `atan :: inline (y: float32) -> float32`
- `atan2 :: (y: float, x: float) -> float`
- `atan2 :: (y: float64, x: float64) -> float64`
- `asin :: (x: float64) -> float64`
- `ASIN_AND_ACOS_ERROR_REPORTING_LIMIT :: 1.000001`
- `acos :: (x: float64) -> float64`
- `exp :: (x: float64) -> float64`
- `pow :: (x: float64, y: float64) -> float64`
- `reduc :: (x: float64) -> float64` — returns `floor(16*x)/16`, the greatest multiple of 1/16 not greater than `x`. For inputs whose scaling by 16 remains finite, the result is at most `x` and differs from it by less than 1/16; negative values round toward negative infinity. Extreme values, infinities, and NaNs receive no separate handling.
- `Matrix2 :: struct`
- `Matrix2_Identity :: Matrix2.{_11=1, _22=1}`
- `Matrix3 :: struct`
- `Matrix3_Identity :: Matrix3.{_11=1, _22=1, _33=1}`
- `Matrix4 :: struct`
- `Matrix4_Identity :: Matrix4.{_11=1, _22=1, _33=1, _44=1}`
- `Matrix4x3 :: struct`
- `Matrix4x3_Identity :: Matrix4x3.{_11=1, _22=1, _33=1}`
- `AnyMatrix3 :: struct`
- `AnyMatrix4x3 :: struct`
- `AnyMatrix2 :: struct`
- `AnyMatrix :: struct`
- `row :: (m: Matrix4, i: int) -> Vector4`
- `column :: (m: Matrix4, i: int) -> Vector4`
- `multiply :: (m: Matrix4, v: Vector4) -> Vector4`
- `multiply :: (m: Matrix4x3, v: Vector4) -> Vector3`
- `multiply :: (m: Matrix2, v: Vector2) -> Vector2`
- `transform_point :: (m: $T/interface AnyMatrix4x3, v: Vector3) -> Vector3`
- `multiply :: (m: $T/interface AnyMatrix3, v: Vector3) -> Vector3`
- `transform_vector :: (m: $T/interface AnyMatrix3, v: Vector3) -> Vector3`
- `multiply :: (m: Matrix2, n: Matrix2) -> Matrix2`
- `multiply :: (m: Matrix3, n: Matrix3) -> Matrix3`
- `multiply :: (m: Matrix4, n: Matrix3) -> Matrix4`
- `multiply :: (m: Matrix4, n: Matrix4) -> Matrix4 #no_alias`
- `multiply :: (m: Matrix4x3, n: Matrix4x3) -> Matrix4x3`
- `multiply :: (m: Matrix4, n: Matrix4x3) -> Matrix4`
- `multiply :: (m: Matrix4x3, n: Matrix4) -> Matrix4`
- `operator * :: (m: Matrix4, v: Vector4) -> Vector4`
- `operator * :: (m: Matrix4x3, v: Vector4) -> Vector3`
- `operator * :: (m: Matrix3, v: Vector3) -> Vector3`
- `operator * :: (a: Matrix4, b: Matrix4) -> Matrix4`
- `operator * :: (a: Matrix4x3, b: Matrix4x3) -> Matrix4x3`
- `operator * :: (a: Matrix4, b: Matrix4x3) -> Matrix4`
- `operator * :: (a: Matrix4x3, b: Matrix4) -> Matrix4`
- `operator * :: (a: Matrix4, b: Matrix3) -> Matrix4`
- `operator * :: (a: Matrix3, b: Matrix3) -> Matrix3`
- `operator * :: (a: Matrix2, b: Matrix2) -> Matrix2`
- `operator * :: (a: Matrix2, b: Vector2) -> Vector2`
- `operator * :: (a: $T/interface AnyMatrix, t: float) -> T #symmetric`
- `operator + :: (a: $T/interface AnyMatrix, b: T) -> T`
- `operator - :: (a: $T/interface AnyMatrix, b: T) -> T`
- `make_matrix3 :: (m: $T/interface AnyMatrix3) -> Matrix3`
- `make_matrix4 :: (m: $T/interface AnyMatrix3) -> Matrix4`
- `transpose :: (m: Matrix4) -> Matrix4`
- `transpose :: (m: Matrix3) -> Matrix3`
- `transpose :: (m: *Matrix4)`
- `transpose :: (m: *Matrix3)`
- `make_scale_matrix4 :: (v: Vector3) -> Matrix4`
- `scale :: (m: $T/interface AnyMatrix3, s: float) -> T`
- `scale :: (m: *$T/interface AnyMatrix3, s: float)`
- `scale :: (m: $T/interface AnyMatrix3, v: Vector3) -> T`
- `scale :: (m: *$T/interface AnyMatrix3, v: Vector3)`
- `make_translation_matrix4 :: (v: Vector3) -> Matrix4`
- `translate :: (using m: Matrix4, t: Vector3) -> Matrix4`
- `translate :: (using m: *Matrix4, t: Vector3)`
- `rotate :: (m: Matrix3, q: Quaternion) -> Matrix3`
- `rotate :: (m: Matrix4, q: Quaternion) -> Matrix4`
- `make_matrix_from_rows :: (xprime: Vector3, yprime: Vector3, zprime: Vector3, $T := Matrix4) -> T`
- `make_matrix_from_columns :: (xprime: Vector3, yprime: Vector3, zprime: Vector3, $T := Matrix4) -> T`
- `make_look_at_matrix :: (viewpoint: Vector3, look_at: Vector3, reference_up_vector: Vector3, $x_is_forward := true) -> Matrix4`
- `orthographic_projection_matrix :: (left: float, right: float, bottom: float, top: float, near: float, far: float, depth_range_01:=false) -> Matrix4`
- `make_projection_matrix :: (fov_vertical: float, aspect_ratio_horizontal_over_vertical: float, z_near: float, z_far: float, x_offset:=0.0, y_offset:=0.0, depth_range_01:=false) -> Matrix4`
- `make_frustum_matrix :: (l: float, r: float, b: float, t: float, n: float, f: float, depth_range_01:=false) -> Matrix4`
- `lerp :: (a: Matrix3, b: Matrix3, t: float) -> Matrix3`
- `inverse :: (a: Matrix3, epsilon := 0.001) -> (success: bool, result: Matrix3)`
- `inverse :: (m: Matrix4) -> (success: bool, result: Matrix4)`
- `inverse_gaussian_elimination :: (m: Matrix4) -> (success: bool, result: Matrix4)`
- `inverse_lengyel :: (m: Matrix4, epsilon := 0.0001) -> (success: bool, result: Matrix4)`
- `adjugate_lengyel :: (m: Matrix4) -> Matrix4`
- `adjugate :: adjugate_lengyel`
- `determinant :: (m: Matrix4) -> float`
- `isometry_inverse :: (m: Matrix4) -> Matrix4`
- `adjoint_transpose :: (m: Matrix3) -> Matrix3`
- `mat_norm :: (M: Matrix3, tpose: bool) -> float`
- `norm_inf :: #bake_arguments mat_norm(tpose=false)`
- `norm_one :: #bake_arguments mat_norm(tpose=true)`
- `make_reflector :: (v: Vector3) -> (u: Vector3)`
- `reflect_cols :: (M: *Matrix3, u: Vector3)`
- `reflect_rows :: (M: *Matrix3, u: Vector3)`
- `polar_decomp :: (M: Matrix3) -> (Q: Matrix3, S: Matrix3, det: float)`
- `rotation_matrix :: ($T: Type, q: Quaternion) -> T`
- `identity_of :: ($T: Type) -> T`
- `identity :: (m: *$T/interface AnyMatrix)`
- `TAU :: cast(float32) 6.283185`
- `TAU64 :: 6.28318530717958648`
- `PI :: cast(float32) 3.1415927`
- `PI64 :: 3.141592653589793`
- `FLOAT16_MAX : float : 65504.0`
- `FLOAT32_MIN :: 0h00800000`
- `FLOAT32_MAX :: 0h7F7FFFFF`
- `FLOAT32_INFINITY :: 0h7F800000`
- `FLOAT32_NAN :: 0h7FBFFFFF`
- `FLOAT64_MIN :: 0h00100000_00000000`
- `FLOAT64_MAX :: 0h7FEFFFFF_FFFFFFFF`
- `FLOAT64_INFINITY :: 0h7FF00000_00000000`
- `FLOAT64_NAN :: 0h7FF7FFFF_FFFFFFFF`
- `S8_MIN :s8: -128`
- `S8_MAX :s8: 127`
- `U8_MAX :u8: 255`
- `S16_MIN :s16: -32768`
- `S16_MAX :s16: 32767`
- `U16_MAX :u16: 0xffff`
- `S32_MIN :s32: 0x8000_0000`
- `S32_MAX :s32: 0x7fff_ffff`
- `U32_MAX :u32: 0xffff_ffff`
- `S64_MIN :s64: 0x8000_0000_0000_0000`
- `S64_MAX :s64: 0x7fff_ffff_ffff_ffff`
- `U64_MAX :u64: 0xffff_ffff_ffff_ffff`
- `Clamp :: inline (pointer: *$T, low: T, high: T)`
- `abs :: inline (x: $T/.[s64, s32, s16, s8]) -> T`
- `is_nan_is_inf :: (x: float64) -> (isnan: bool, isinf: bool)`
- `isnan :: (x: float64) -> bool`
- `isinf :: (x: float64) -> bool`
- `is_finite :: (x: float64) -> bool`
- `is_nan_is_inf :: (x: float32) -> (isnan: bool, isinf: bool)`
- `isnan :: (x: float32) -> bool`
- `isinf :: (x: float32) -> bool`
- `is_finite :: (x: float) -> bool`
- `abs :: inline (x: float64) -> float64`
- `abs :: inline (x: float32) -> float32`
- `frac :: inline (x: $T) -> T #modify`
- `fmod_cycling :: inline (x: float64, y: float64) -> float64`
- `fmod_cycling :: inline (x: float, divisor: float) -> float`
- `tan :: inline (theta: float) -> float`
- `tan :: (x: float64) -> float64`
- `acos :: inline (theta: float) -> float`
- `asin :: inline (theta: float) -> float`
- `sqrt :: inline (x: float) -> float`
- `sqrt :: (x: float64) -> float64`
- `pow :: inline (x: float, power: float) -> float`
- `exp :: inline (x: float) -> float`
- `log :: inline (x: float) -> float`
- `log :: (x: float64) -> float64`
- `saturate :: (f: float) -> float`
- `saturate :: (f: float64) -> float64`
- `square :: (a : $T) -> T`
- `half_to_float :: (h: u16) -> float`
- `Vector2 :: struct`
- `Vector3 :: struct`
- `Vector4 :: struct`
- `Quaternion :: struct`
- `Complex :: #type,isa Vector2`
- `dot :: dot_product`
- `dot_product :: (a: Vector2, b: Vector2) -> float`
- `dot_product :: (a: Vector3, b: Vector3) -> float`
- `dot_product :: (a: Vector4, b: Vector4) -> float`
- `dot_product :: (a: Quaternion, b: Quaternion) -> float`
- `length :: (a: Vector2) -> float`
- `length :: (a: Vector3) -> float`
- `length :: (a: Vector4) -> float`
- `length :: (a: Quaternion) -> float`
- `length_squared :: inline (a: Vector2) -> float`
- `length_squared :: inline (a: Vector3) -> float`
- `length_squared :: inline (a: Vector4) -> float`
- `make_vector2 :: (x: float) -> Vector2`
- `make_vector3 :: (x: float) -> Vector3`
- `make_vector3 :: (c: [3] float) -> Vector3`
- `make_vector3 :: (using v: Vector2, z: float) -> Vector3`
- `make_vector4 :: (x: float) -> Vector4`
- `make_vector4 :: (using v: Vector2, z: float, w: float) -> Vector4`
- `make_vector4 :: (using v: Vector3, w: float) -> Vector4`
- `xy :: (x: float, y: float) -> Vector2`
- `xy :: (x: float) -> Vector2`
- `xyz :: (x: float, y: float, z: float) -> Vector3`
- `xyz :: (x: float) -> Vector3`
- `xyz :: (c: [3] float) -> Vector3`
- `xyz :: (using v: Vector2, z: float) -> Vector3`
- `xyzw :: (x: float, y: float, z: float, w: float) -> Vector4`
- `xyzw :: (x: float) -> Vector4`
- `xyzw :: (using v: Vector2, z: float, w: float) -> Vector4`
- `xyzw :: (using v: Vector3, w: float) -> Vector4`
- `rgb :: xyz`
- `rgba :: xyzw`
- `normalize_or_identity :: (using q: *Quaternion) -> float`
- `abs :: inline (using v: Vector2) -> Vector2`
- `abs :: inline (using v: Vector3) -> Vector3`
- `abs :: inline (using v: Vector4) -> Vector4`
- `min :: inline (v1: Vector2, v2: Vector2) -> Vector2`
- `min :: inline (v1: Vector3, v2: Vector3) -> Vector3`
- `min :: inline (v1: Vector4, v2: Vector4) -> Vector4`
- `max :: inline (v1: Vector2, v2: Vector2) -> Vector2`
- `max :: inline (v1: Vector3, v2: Vector3) -> Vector3`
- `max :: inline (v1: Vector4, v2: Vector4) -> Vector4`
- `lerp :: inline (a: float, b: float, t: float) -> float`
- `lerp :: inline (a: float64, b: float64, t: float64) -> float64`
- `lerp :: (a: $R, b: R, t: float) -> R #modify`
- `lerp :: (a: Quaternion, b: Quaternion, t: float) -> Quaternion`
- `lerp :: (a: Vector2, b: Vector2, t: float) -> Vector2`
- `lerp :: (a: Vector3, b: Vector3, t: float) -> Vector3`
- `lerp :: (a: Vector4, b: Vector4, t: float) -> Vector4`
- `nlerp :: inline (p0: Quaternion, p1: Quaternion, fraction: float) -> Quaternion`
- `slerp :: (start: Quaternion, end: Quaternion, t: float) -> Quaternion`
- `multiply :: (v: Vector2, f: float) -> Vector2`
- `multiply :: (v: Vector3, f: float) -> Vector3`
- `multiply :: (v: Vector4, f: float) -> Vector4`
- `distance :: (a: Vector4, b: Vector4) -> float`
- `distance_squared :: inline (a: Vector4, b: Vector4) -> float`
- `distance :: (a: Vector3, b: Vector3) -> float`
- `distance_squared :: inline (a: Vector3, b: Vector3) -> float`
- `distance :: (a: Vector2, b: Vector2) -> float`
- `distance_squared :: inline (a: Vector2, b: Vector2) -> float`
- `AnyVector2 :: struct`
- `normalize :: (v: *$T/MyVectors, epsilon := 0.0, fallback := T.{}) -> float`
- `normalize :: (v: $T/MyVectors, epsilon := 0.0, fallback := T.{}) -> T`
- `unit_vector :: (v: Vector4) -> Vector4`
- `unit_vector :: (v: Vector3) -> Vector3`
- `unit_vector :: (v: Vector2) -> Vector2`
- `rotate :: (v: *Vector2, theta: float)`
- `rotate :: (v: Vector2, theta: float) -> Vector2`
- `cross :: cross_product`
- `cross_product :: (a: Vector3, b: Vector3) -> Vector3`
- `rotate :: (v: Vector3, q: Quaternion) -> Vector3`
- `rotate :: (v: *Vector3, q: Quaternion)`
- `saturate :: (a: Vector2) -> Vector2`
- `saturate :: (a: Vector3) -> Vector3`
- `saturate :: (a: Vector4) -> Vector4`
- `operator + :: (a: Vector2, b: Vector2) -> Vector2`
- `operator - :: (a: Vector2, b: Vector2) -> Vector2`
- `operator - :: (a: Vector2) -> Vector2`
- `operator * :: (a: Vector2, b: Vector2) -> Vector2`
- `operator * :: (a: Vector2, k: float) -> Vector2 #symmetric`
- `operator / :: (a: Vector2, b: Vector2) -> Vector2`
- `operator / :: (a: Vector2, k: float) -> Vector2`
- `operator / :: (k: float, a: Vector2) -> Vector2`
- `set_from_axis_and_angle :: (using q: *Quaternion, v: Vector3, theta: float)`
- `set_from_axis_and_angle :: inline (q: *Quaternion, x: float, y: float, z: float, theta: float)`
- `get_axis_and_angle :: (using q: Quaternion) -> x: float64, y: float64, z: float64, theta: float64`
- `conjugate :: (using q: Quaternion) -> Quaternion`
- `set_rotation :: (using m: *$T/interface AnyMatrix3, q: Quaternion)`
- `get_rotation :: (using m: $T/interface AnyMatrix3) -> Quaternion`
- `make_an_orthonormal_basis :: (x_axis: Vector3) -> (y_axis: Vector3, z_axis: Vector3)`
- `make_an_orthonormal_basis :: (x_axis: Vector3, approximate_axis: Vector3) -> (y_axis: Vector3, z_axis: Vector3)`
- `Plane3 :: struct`
- `make_plane :: (a: float, b: float, c: float, d: float) -> Plane3`
- `make_plane :: (v: Vector3, d: float) -> Plane3`
- `make_plane :: (v: Vector3, x: Vector3) -> Plane3`
- `plane_from_point_and_normal :: (point: Vector3, normal: Vector3) -> Plane3`
- `make_plane :: (v: Vector4) -> Plane3`
- `get_normal :: (p: Plane3) -> Vector3`
- `normalize :: (using p: Plane3) -> Plane3`
- `normalized_plane_from_points :: (p0: Vector3, p1: Vector3, p2: Vector3) -> Plane3`
- `plane_dot :: (plane: Plane3, v: Vector3) -> float #symmetric`
- `print_matrix :: (builder: *String_Builder, any: Any, user_data: *void) -> bool`
- `math_struct_printer :: (builder: *String_Builder, any: Any, user_data: *void) -> bool`

- `SQRT2 :: 1.41421356237309504880`
- `SQRTH :: 0.70710678118654752440`
- `cot :: (x: float64) -> float64`
- `frexp :: (x: float64) -> (result: float64, pow2: s32)`
- `ldexp :: (x: float64, pw2: s32) -> float64`
- `signbit :: inline (x: float64) -> u64`

## Conventions

Components are 32-bit `float`. Coordinate systems are right-handed and angles are radians. Matrices are stored row-major (`_rc` is row `r`, column `c`) and act on column vectors, with the translation in the last column; `make_projection_matrix` and `make_frustum_matrix` build a camera that looks along -z with clip depth -1..1 (0..1 with `depth_range_01`), and `make_look_at_matrix` pairs with them when `x_is_forward` is false. `Quaternion` stores the vector part first and the scalar `w` last, and `Quaternion.{}` is the identity. `Complex` is `#type,isa Vector2`. `plane_dot` is `#symmetric`, so the point can come before the plane. The doc comments in `stdlib/Math/module.jai` carry the per-procedure behavior and examples.

## Public type members

#### `Plane3` (struct)

- `a, b, c, d : float`
- `#overlay (a) #as vector4: Vector4`
- `#overlay (a) normal: Vector3`

#### `Vector2` (struct)

- `x: float`
- `y: float`
- Overlay `component: [2]float` starts at `x`.

#### `Vector3` (struct)

- `x: float`
- `y: float`
- `z: float`
- Overlays `xy: Vector2` at `x`, `yz: Vector2` at `y`, and `component: [3]float` at `x`.

#### `Vector4` (struct)

- `x: float`
- `y: float`
- `z: float`
- `w: float`
- Overlays `xy: Vector2` at `x`, `yz: Vector2` at `y`, `zw: Vector2` at `z`, `xyz: Vector3` at `x`, `yzw: Vector3` at `y`, and `component: [4]float` at `x`.

#### `Quaternion` (struct)

- `x: float`
- `y: float`
- `z: float`
- `w := 1.0`
- Overlays `xyz: Vector3` and `component: [4]float`, both starting at `x`.

#### `Matrix2` (struct)

- `_11, _12, _21, _22: float`
- Overlays `v: [2]Vector2`, `coef: [2][2]float`, and `floats: [4]float` start at `_11`.
- `Row_Type :: Vector2`; `Column_Type :: Vector2`; `IsMatrixFromMathModule :: true`.

#### `Matrix3` (struct)

- `_11, _12, _13, _21, _22, _23, _31, _32, _33: float`
- Overlays `v: [3]Vector3`, `coef: [3][3]float`, and `floats: [9]float` start at `_11`.
- `Row_Type :: Vector3`; `Column_Type :: Vector3`; `IsMatrixFromMathModule :: true`.

#### `Matrix4` (struct)

- `_11, _12, _13, _14, _21, _22, _23, _24, _31, _32, _33, _34, _41, _42, _43, _44: float`
- Overlays `mat4x3: Matrix4x3`, `v: [4]Vector4`, `coef: [4][4]float`, and `floats: [16]float` start at `_11`.
- `Row_Type :: Vector4`; `Column_Type :: Vector4`; `IsMatrixFromMathModule :: true`.

#### `Matrix4x3` (struct)

- `_11, _12, _13, _14, _21, _22, _23, _24, _31, _32, _33, _34: float`
- Overlays `v: [3]Vector4`, `coef: [3][4]float`, and `floats: [12]float` start at `_11`.
- `Row_Type :: Vector4`; `Column_Type :: Vector3`; `IsMatrixFromMathModule :: true`.

#### Matrix interface shapes

- `AnyMatrix3`: `_11, _12, _13, _21, _22, _23, _31, _32, _33: float`. `get_rotation` also reads a `coef` matrix view not represented by this interface, so accepted matrix types must provide that compatible view.
- `AnyMatrix4x3`: `_11, _12, _13, _14, _21, _22, _23, _24, _31, _32, _33, _34: float`.
- `AnyMatrix2`: `_11, _12, _21, _22: float`.
- `AnyMatrix`: `IsMatrixFromMathModule :: true`.
- `AnyVector2`: `x: float`, `y: float`.

## Behavioral contract and constraints

The module groups scalar numeric constants and helpers for common math operations, geometry, and interpolation. Functions generally return numeric values directly; domain/range behavior is operation-specific and not uniformly checked. Callers should provide finite values and valid ranges where required; no global NaN/error policy is specified.

The typed initialized constants use Jai's `name : Type : initializer` declaration form and are immutable; this includes `LOSSTH`, `FLOAT16_MAX`, and the typed integer limit values. Declarations using `::` are also constants. `LOSSTH` is read by the Cephes trigonometric range-reduction path; callers cannot change it to alter later calls. The limit values describe numeric bounds and do not redefine the representable range of their machine types.

### Operation-level behavior

| Operation group | Observable behavior |
| --- | --- |
| `Clamp` | Mutates the pointed-to value only when below `low` or above `high`; callers are responsible for providing an ordered interval. |
| Integer `abs` | Supports signed 8/16/32/64-bit integers. The minimum two's-complement value has no representable positive magnitude and triggers an assertion. Float `abs` clears the sign bit. |
| `isnan`, `isinf`, `is_finite` | Classify IEEE single/double precision encodings. NaN and infinity are non-finite; finite includes subnormals and signed zero. |
| `frac` | The `#modify` transformation is available for `float32` and `float64` and computes `x - floor(x)`, so finite results lie in `[0, 1)`; negative values yield a positive fractional part. |
| `fmod_cycling` | Returns `x - floor(x / divisor) * divisor`, giving floor-based cycling rather than C remainder behavior. A zero divisor logs an error and returns positive infinity. Non-finite inputs have no additional contract. |
| `saturate` | Clamps to `[0, 1]`; `lerp` is linear interpolation and does not clamp `t`, so values outside `[0,1]` extrapolate. |
| Vector helpers | `dot_product`, `length`, and `length_squared` use all components; `min`, `max`, and `abs` operate componentwise. `normalize` treats `epsilon` as a threshold on vector length (not squared length): when `length <= epsilon`, it writes `fallback` and returns that length; otherwise it normalizes in place or returns a normalized copy, depending on overload. `unit_vector` returns a zero vector unchanged for zero input. `normalize_or_identity` for quaternions mutates the value, returns its original length, and maps a zero quaternion to identity. |
| Quaternion interpolation | `nlerp` linearly interpolates and normalizes. `slerp` expects unit-length inputs; its result is not specified for invalid/non-unit inputs. |
| Matrix/vector operations | Matrix members are laid out as rows. Matrix-times-vector multiplies each row by the vector; `Matrix4x3 * Vector4` returns a `Vector3`, while `transform_point` takes a `Vector3` and applies the affine translation. Inverse routines that return `(success, result)` require callers to check `success` before using the result. |
| Inverse edge cases | `inverse(Matrix3, epsilon)` returns `(false, input)` when `-epsilon < determinant < epsilon`; equality at either threshold is not rejected by this test. The default epsilon is `0.001`. The `Matrix4` inverse and `inverse_gaussian_elimination` have no epsilon parameter; they return `(false, Matrix4_Identity)` when their pivot/singularity check fails. |
| `half_to_float` | Converts the bit pattern of a 16-bit float into `float`; signed zero, infinities, and NaN encodings are retained as corresponding single-precision categories. |

Several transcendental routines delegate to host/compiler math operations, and the module does not provide one uniform domain-error policy. Callers should check domains and inverse success where those are relevant.

`Vector2`, `Vector3`, and `Vector4` define componentwise vector/vector `+`, `-`, `*`, and `/`, plus unary negation. Multiplication accepts a scalar on either side; division accepts either a scalar denominator or numerator. There are no scalar `+` or `-` overloads in this interface.

The three vector sizes also provide `multiply(vector, scalar)`, `distance_squared`, and `unit_vector`. For example, `Vector4.{1, 2, 3, 4} * Vector4.{5, 6, 7, 8}` returns `Vector4.{5, 12, 21, 32}`; use `dot_product` for a scalar dot product.

Vector arithmetic lives in `stdlib/internal/numerics/math_vectors.jai` and depends on the vector types and scalar helpers loaded by `Math`, with no configuration flags. Keep the three vector overload families in step and extend `stdlib/Math/tests/vector-arithmetic.jai` when changing them. `Vector3` additionally multiplies by a `float64` scale factor on either side (and in `*=`), narrowing it to `float`; no other vector type or operator takes a `float64`. Quaternion multiplication has its own rotation-composition meaning and is implemented separately in `math_quaternion.jai`.

### Matrix operators

Matrix multiplication overloads are public for `Matrix2*Matrix2`,
`Matrix3*Matrix3`, `Matrix4*Matrix3`, `Matrix4*Matrix4`,
`Matrix4x3*Matrix4x3`, `Matrix4*Matrix4x3`, and `Matrix4x3*Matrix4`;
`Matrix4*Matrix4` has `#no_alias`. Matrix/vector overloads are
`Matrix2*Vector2`, `Matrix3*Vector3`, `Matrix4*Vector4`, and
`Matrix4x3*Vector4` (which returns `Vector3`). These operators follow the
corresponding `multiply` order: `a*b` is the standard row-by-column product
with `a` on the left and `b` on the right (when applied to a column vector,
`b` acts first). The operation is not commutative. In particular, the two mixed `Matrix4` /
`Matrix4x3` signatures are distinct and must not be reversed or collapsed.

`Matrix2`, `Matrix3`, `Matrix4`, and `Matrix4x3` all support `* float` through
the `AnyMatrix` interface with `#symmetric`, so a scalar may appear on either
side and each stored coefficient is scaled. The same interface supplies
`matrix + same_matrix_type` and `matrix - same_matrix_type`; these are
coefficient-wise operations and have no symmetric flag. The matrix/vector `*`
operators are the named `multiply` operations, while the vector/vector `*`
operators are component-wise (Hadamard); the shared operator symbol therefore
has overload-specific meaning determined by operand types. No matrix/matrix
overload is marked `#symmetric`.

## Portability notes

A bare `sqrt` call resolves to a compiler intrinsic here. The official compiler reports it as an undeclared identifier unless the program names the module, so call it as `Math.sqrt` (or `using Math`) in code that must build with both.
