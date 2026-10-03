# Float16

## Public declaration inventory

Visible public declarations and module-conditional variants are listed below. Procedure bodies and aggregate initializers are omitted; complete public type members are listed separately. The module parameter is `ASSERT_ON_OVERFLOW := false`.

- `Float16 :: #type,isa u16`
- `Vector2_Float16 :: struct`
- `Vector3_Float16 :: struct`
- `Vector4_Float16 :: struct`
- `Matrix3_Float16 :: struct`
- `vector2_float32 :: (v: Vector2_Float16) -> Vector2`
- `vector2_float16 :: (v: Vector2) -> Vector2_Float16`
- `vector3_float32 :: (v: Vector3_Float16) -> Vector3`
- `vector4_float16 :: (v: Vector4) -> Vector4_Float16`
- `vector3_float16 :: (v: Vector3) -> Vector3_Float16`
- `matrix3_float32 :: (s: Matrix3_Float16) -> Matrix3`
- `matrix3_float16 :: (s: Matrix3) -> Matrix3_Float16`
- `float32_to_float16 :: float_to_half_fast3`
- `float16_to_float32 :: half_to_float_fast4`
- `float_to_half_fast3 :: (_f: float32) -> Float16`
- `half_to_float_fast4 :: (h: Float16) -> float32`

## Public type members

#### `Vector2_Float16` (struct)

- `x: Float16`
- `y: Float16`

#### `Vector3_Float16` (struct)

- `x: Float16`
- `y: Float16`
- `z: Float16`

#### `Vector4_Float16` (struct)

- `x: Float16`
- `y: Float16`
- `z: Float16`
- `w: Float16`

#### `Matrix3_Float16` (struct)

- `_11: Float16`
- `_12: Float16`
- `_13: Float16`
- `_21: Float16`
- `_22: Float16`
- `_23: Float16`
- `_31: Float16`
- `_32: Float16`
- `_33: Float16`
- `floats: [9]Float16` overlays the same storage beginning at `_11`.

## Behavioral contract and constraints

The module defines a `u16`-backed half-float type and vector/matrix conversions between 16-bit storage and 32-bit Jai math types. Conversion preserves sign, handles infinities and NaNs, and clamps finite overflow to signed infinity; when `ASSERT_ON_OVERFLOW` is enabled, finite overflow also asserts. Conversion is documented as matching round-to-nearest-even SSE arithmetic for every `float32` input; behavior under other floating-point rounding modes is not guaranteed. Matrix conversion covers the nine overlaid elements. This is a starting-point utility, not a complete float16 library.
