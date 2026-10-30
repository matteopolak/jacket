// Converts one sRGB-encoded value in [0, 1] to linear light with the exact piecewise sRGB curve: a straight segment `x / 12.92` up to 0.04045 and `((x + 0.055) / 1.055) ^ 2.4` above it. 0 gives 0 and 1 gives 1. Values outside [0, 1] are not clamped: the curve is evaluated as written, so a negative input stays negative and a large one is extrapolated.
//
// ```jai
// #import "Math";
// assert(abs(srgb_to_linear(0.5) - 0.21404114) < 0.00001);
// assert(srgb_to_linear(0.0) == 0 && abs(srgb_to_linear(1.0) - 1) < 0.00001);
// ```
srgb_to_linear :: inline (x: float) -> float {
    if x <= SRGB_LINEAR_LIMIT return x / SRGB_LINEAR_SLOPE;
    return pow((x + SRGB_OFFSET) / SRGB_SCALE, SRGB_GAMMA);
}

// Converts one linear-light value in [0, 1] to sRGB encoding with the exact piecewise curve (the inverse of `srgb_to_linear`): `x * 12.92` up to 0.0031308 and `1.055 * x ^ (1 / 2.4) - 0.055` above it. Values outside [0, 1] are not clamped; clamp first when the result feeds an 8-bit store.
//
// ```jai
// #import "Math";
// assert(abs(linear_to_srgb(0.21404114) - 0.5) < 0.00001);
// x := linear_to_srgb(srgb_to_linear(0.3));
// assert(abs(x - 0.3) < 0.00001);
// ```
linear_to_srgb :: (x: float) -> float {
    if x <= LINEAR_SRGB_LIMIT return x * SRGB_LINEAR_SLOPE;
    return SRGB_SCALE * pow(x, 1.0 / SRGB_GAMMA) - SRGB_OFFSET;
}

// The fast path uses a piecewise linear interpolant of the exact transfer curve.
// Sampling in encoded space concentrates linear-domain samples near black.
// A faster `srgb_to_linear` for values in [0, 1]: linear interpolation over a 64-segment table of the exact curve, built at compile time, so there is no `pow` call at run time. The error is below about 0.002 in the linear result. Inputs of 1 or more fall back to the exact curve, and the linear segment below 0.04045 is exact.
//
// ```jai
// #import "Math";
// x := srgb_to_linear_fast(0.5);
// assert(abs(x - srgb_to_linear(0.5)) < 0.002);
// ```
srgb_to_linear_fast :: inline (x: float) -> float {
    if x <= SRGB_LINEAR_LIMIT return x / SRGB_LINEAR_SLOPE;
    if x >= 1 return srgb_to_linear(x);

    index := cast(int) (x * 64);
    lo := cast(float) index / 64;
    a := srgb_samples[index];
    b := srgb_samples[index + 1];

    return a + (b - a) * (x - lo) * 64;
}

// A faster `linear_to_srgb` for values in [0, 1]: a binary search over the same 64-segment table, inverted by linear interpolation, so there is no `pow` call at run time. The error is below about 0.003 in the encoded result. Inputs of 1 or more fall back to the exact curve, and the linear segment below 0.0031308 is exact.
//
// ```jai
// #import "Math";
// x := linear_to_srgb_fast(0.5);
// assert(abs(x - linear_to_srgb(0.5)) < 0.003);
// ```
linear_to_srgb_fast :: inline (x: float) -> float {
    if x <= LINEAR_SRGB_LIMIT return x * SRGB_LINEAR_SLOPE;
    if x >= 1 return linear_to_srgb(x);

    // Binary search avoids evaluating a transcendental operation at runtime once
    // the transfer table is initialized by compile-time execution.
    low: int = 0;
    high: int = 64;

    while high - low > 1 {
        mid := (low + high) / 2;
        if x < srgb_samples[mid] high = mid;
        else low = mid;
    }

    fraction := (x - srgb_samples[low]) / (srgb_samples[high] - srgb_samples[low]);

    return (cast(float) low + fraction) / 64;
}

// Approximates `srgb_to_linear` by squaring each channel, a gamma of 2.0 instead of the sRGB curve. Cheap, and within a few percent of the real curve in the middle of the range (0.5 gives 0.25 instead of 0.214), but noticeably off near black.
//
// ```jai
// #import "Math";
// assert(srgb_to_linear_pow2(xyz(0.5, 1, 0)) == xyz(0.25, 1, 0));
// ```
srgb_to_linear_pow2 :: inline (c: Vector3) -> Vector3 {
    return .{ c.x * c.x, c.y * c.y, c.z * c.z };
}

// Approximates `linear_to_srgb` by taking the square root of each channel, a gamma of 2.0 instead of the sRGB curve (the inverse of `srgb_to_linear_pow2`). A negative channel gives NaN.
//
// ```jai
// #import "Math";
// assert(linear_to_srgb_sqrt(xyz(0.25, 1, 0)) == xyz(0.5, 1, 0));
// ```
linear_to_srgb_sqrt :: inline (c: Vector3) -> Vector3 {
    return .{ sqrt(c.x), sqrt(c.y), sqrt(c.z) };
}

// The `Vector3` form converts the three color channels, each as the scalar form does.
srgb_to_linear :: inline (c: Vector3) -> Vector3 {
    return .{ srgb_to_linear(c.x), srgb_to_linear(c.y), srgb_to_linear(c.z) };
}

// The `Vector3` form converts the three color channels, each as the scalar form does.
linear_to_srgb :: inline (c: Vector3) -> Vector3 {
    return .{ linear_to_srgb(c.x), linear_to_srgb(c.y), linear_to_srgb(c.z) };
}

// The `Vector3` form converts the three color channels, each as the scalar form does.
srgb_to_linear_fast :: inline (c: Vector3) -> Vector3 {
    return .{ srgb_to_linear_fast(c.x), srgb_to_linear_fast(c.y), srgb_to_linear_fast(c.z) };
}

// The `Vector3` form converts the three color channels, each as the scalar form does.
linear_to_srgb_fast :: inline (c: Vector3) -> Vector3 {
    return .{ linear_to_srgb_fast(c.x), linear_to_srgb_fast(c.y), linear_to_srgb_fast(c.z) };
}

// The `Vector4` form squares the red, green and blue channels and leaves alpha (`w`) untouched.
srgb_to_linear_pow2 :: inline (c: Vector4) -> Vector4 {
    return .{ c.x * c.x, c.y * c.y, c.z * c.z, c.w };
}

// The `Vector4` form takes the square root of the red, green and blue channels and leaves alpha (`w`) untouched.
linear_to_srgb_sqrt :: inline (c: Vector4) -> Vector4 {
    return .{ sqrt(c.x), sqrt(c.y), sqrt(c.z), c.w };
}

// The `Vector4` form converts the red, green and blue channels and leaves alpha (`w`) untouched.
srgb_to_linear :: inline (c: Vector4) -> Vector4 {
    return .{ srgb_to_linear(c.x), srgb_to_linear(c.y), srgb_to_linear(c.z), c.w };
}

// The `Vector4` form converts the red, green and blue channels and leaves alpha (`w`) untouched.
linear_to_srgb :: inline (c: Vector4) -> Vector4 {
    return .{ linear_to_srgb(c.x), linear_to_srgb(c.y), linear_to_srgb(c.z), c.w };
}

// The `Vector4` form converts the red, green and blue channels and leaves alpha (`w`) untouched.
srgb_to_linear_fast :: inline (c: Vector4) -> Vector4 {
    return .{ srgb_to_linear_fast(c.x), srgb_to_linear_fast(c.y), srgb_to_linear_fast(c.z), c.w };
}

// The `Vector4` form converts the red, green and blue channels and leaves alpha (`w`) untouched.
linear_to_srgb_fast :: inline (c: Vector4) -> Vector4 {
    return .{ linear_to_srgb_fast(c.x), linear_to_srgb_fast(c.y), linear_to_srgb_fast(c.z), c.w };
}

#scope_module
make_srgb_samples :: () -> [65] float {
    result: [65] float;
    for i: 0..64 result[i] = srgb_to_linear(cast(float) i / 64);
    return result;
}

srgb_samples :: #run make_srgb_samples();

#scope_file

// sRGB transfer curve parameters.
SRGB_LINEAR_LIMIT :: 0.04045;
LINEAR_SRGB_LIMIT :: 0.0031308;
SRGB_LINEAR_SLOPE :: 12.92;
SRGB_SCALE :: 1.055;
SRGB_OFFSET :: 0.055;
SRGB_GAMMA :: 2.4;
