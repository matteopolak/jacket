# Srgb


## Public declaration inventory

Only names visible through the module’s exported scope are listed. Declarations from conditional branches retain their condition label; implementation-local declarations are omitted. Aggregate fields and enum members are recorded in the dedicated type-shape section.

- `srgb_to_linear :: inline (x: float) -> float`
- `linear_to_srgb :: (x: float) -> float`
- `srgb_to_linear_fast :: inline (x: float) -> float`
- `linear_to_srgb_fast :: inline (x: float) -> float`
- `srgb_to_linear_pow2 :: inline (c: Vector3) -> Vector3`
- `srgb_to_linear_fast :: inline (c: Vector3) -> Vector3`
- `srgb_to_linear :: inline (c: Vector3) -> Vector3`
- `linear_to_srgb_sqrt :: inline (c: Vector3) -> Vector3`
- `linear_to_srgb_fast :: inline (c: Vector3) -> Vector3`
- `linear_to_srgb :: inline (c: Vector3) -> Vector3`
- `srgb_to_linear_pow2 :: inline (c: Vector4) -> Vector4`
- `srgb_to_linear_fast :: inline (c: Vector4) -> Vector4`
- `srgb_to_linear :: inline (c: Vector4) -> Vector4`
- `linear_to_srgb_sqrt :: inline (c: Vector4) -> Vector4`
- `linear_to_srgb_fast :: inline (c: Vector4) -> Vector4`
- `linear_to_srgb :: inline (c: Vector4) -> Vector4`


## Behavioral contract

Converts scalar, `Vector3`, and `Vector4` channel values between sRGB-encoded and linear-light space. Exact transfer functions and faster approximations are exposed separately; alpha in vector conversions is passed through unchanged. The power/square-root approximations trade accuracy for speed and should not be substituted when exact transfer behavior is required. Inputs are expected in the ordinary normalized channel domain; clamping behavior is not promised. Depends on `Math` vectors and floating-point operations.
