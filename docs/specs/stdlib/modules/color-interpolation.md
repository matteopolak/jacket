# Color_Interpolation

## Public declaration inventory

Visible public declarations and module-conditional variants are listed below. Procedure bodies and aggregate initializers are omitted; complete public type members are listed separately.

- `Interpolation :: struct`
- `Color_Keyframe :: struct`
- `generate_color_samples :: (sorted: [] *$T/Color_Keyframe, interp: Interpolation, output_samples: [] Vector4)`
- `make_colors :: (sorted: [] *$T/Color_Keyframe, space: type_of(Interpolation.color_space)) -> [] Vector3`
- `interpolate_one_sample :: inline (i0: int, t: float, interp: Interpolation, sorted: [] *$T/Color_Keyframe, temp_colors: [] Vector3) -> (rgba: Vector4)`
- `pow :: inline (a: Vector3, power: float) -> Vector3`
- `hermite :: (a: float, b: float, t: float) -> float`
- `hermite :: (a: Vector3, b: Vector3, t: float) -> Vector3`
- `easeInOutQuad :: inline (t: float) -> float`
- `hsv_to_rgb :: (hsv: Vector3) -> Vector3`
- `hsv_to_rgb :: (cx: float, cy: float, cz: float) -> (r: float, g: float, b: float)`
- `rgb_to_hsv :: (rgb: Vector3) -> Vector3`
- `rgb_to_hsv :: (r: float, g: float, b: float) -> (h: float, s: float, v: float)`
- `linear_srgb_to_oklab :: (c: Vector3) -> Vector3`
- `oklab_to_linear_srgb :: (c: Vector3) -> Vector3`
- `GAMMA :: 2.2`

## Public type members

#### `Interpolation` (struct)

- `color_space: enum u8 { RGB :: 0; HSV :: 1; OKLAB :: 2; }`
- `geometry: enum u8 { LINEAR :: 0; QUADRIC :: 1; HERMITE :: 2; }`
- `wrapping := false`

#### `Color_Keyframe` (struct)

- `color_rgb: Vector4`
- `t: float`

## Behavioral contract and constraints

`Color_Keyframe.color_rgb` stores sRGB-like RGBA values and `t` is the key's parameter. `generate_color_samples` assumes keyframes are sorted by increasing `t`; it does not sort or validate the input. For an output slice of length greater than one, sample parameters are evenly spaced from `0` through `1`, inclusive. An empty output slice is unchanged. With no keyframes, output remains unchanged. With exactly one output slot and at least one keyframe, that slot receives the first key's RGBA value directly, without the normal color-space/gamma conversion.

With multiple output slots and at least one key, the selected segment is evaluated at each sample. Without wrapping, indices at either end use the first or last available segment, but `t` is not clamped to the keyframe range; samples outside it can extrapolate. With wrapping enabled, segment lookup crosses the last-to-first boundary by one parameter period. Duplicate neighboring key times use a unit denominator for interpolation rather than producing a divide-by-zero result. Input ordering and the intended key-time interval are caller preconditions.

`LINEAR` uses linear interpolation, `QUADRIC` uses an ease-in/ease-out quadratic parameter, and `HERMITE` uses the two-endpoint cubic smoothstep curve; there are no tangent parameters. These geometry choices apply to alpha as well as color. RGB and OKLab interpolation operate on gamma-adjusted color components, with `GAMMA == 2.2`; the conversion is an approximation and not the piecewise sRGB transfer function. HSV conversion uses hue in degrees, and interpolation takes the shorter hue arc across the 0/360 boundary. The OKLab-to-linear-sRGB conversion clamps output RGB components to `[0, 1]`; interpolation helpers do not promise general clamping for other spaces or for alpha.

`output_samples` is caller-allocated and written in place. `make_colors` returns a newly allocated array whose lifetime follows the active allocator; no explicit deinitializer is provided by this API. `interpolate_one_sample` requires at least one keyframe and a `temp_colors` array corresponding to the keyframe slice; invalid indices, mismatched counts, or unsupported enum values are not a recoverable error contract. Unsupported color-space values are logged by the sampling routine; geometry values outside the declared enum have no specified result.
