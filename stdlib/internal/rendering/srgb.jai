// Converts an sRGB-encoded value in [0, 1] to linear light, using the exact sRGB curve. Values outside [0, 1] are not clamped.
//
// ```jai
// #import "Math";
// assert(abs(srgb_to_linear(0.5) - 0.21404114) < 0.00001);
// assert(srgb_to_linear(0.0) == 0);
// ```
srgb_to_linear :: inline (x: float) -> float {
    if x <= SRGB_LINEAR_LIMIT return x / SRGB_LINEAR_SLOPE;
    return pow((x + SRGB_OFFSET) / SRGB_SCALE, SRGB_GAMMA);
}

// Converts linear light in [0, 1] to sRGB encoding, the inverse of [srgb_to_linear]. Values outside [0, 1] are not clamped; clamp first when the result goes into 8 bits.
//
// ```jai
// #import "Math";
// assert(abs(linear_to_srgb(0.21404114) - 0.5) < 0.00001);
// assert(abs(linear_to_srgb(srgb_to_linear(0.3)) - 0.3) < 0.00001);
// ```
linear_to_srgb :: (x: float) -> float {
    if x <= LINEAR_SRGB_LIMIT return x * SRGB_LINEAR_SLOPE;
    return SRGB_SCALE * pow(x, 1.0 / SRGB_GAMMA) - SRGB_OFFSET;
}

// A faster [srgb_to_linear] for values in [0, 1], using a lookup table instead of `pow`. Off by less than 0.002.
//
// ```jai
// #import "Math";
// assert(abs(srgb_to_linear_fast(0.5) - srgb_to_linear(0.5)) < 0.002);
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

// A faster [linear_to_srgb] for values in [0, 1], using a lookup table instead of `pow`. Off by less than 0.003.
//
// ```jai
// #import "Math";
// assert(abs(linear_to_srgb_fast(0.5) - linear_to_srgb(0.5)) < 0.003);
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

// A rough [srgb_to_linear] that squares each channel (gamma 2.0). Cheap, and close in the middle of the range, but off near black.
//
// ```jai
// #import "Math";
// assert(srgb_to_linear_pow2(xyz(0.5, 1, 0)) == xyz(0.25, 1, 0));
// ```
srgb_to_linear_pow2 :: inline (c: Vector3) -> Vector3 {
    return .{ c.x * c.x, c.y * c.y, c.z * c.z };
}

// A rough [linear_to_srgb] that takes the square root of each channel, the inverse of [srgb_to_linear_pow2]. A negative channel gives NaN.
//
// ```jai
// #import "Math";
// assert(linear_to_srgb_sqrt(xyz(0.25, 1, 0)) == xyz(0.5, 1, 0));
// ```
linear_to_srgb_sqrt :: inline (c: Vector3) -> Vector3 {
    return .{ sqrt(c.x), sqrt(c.y), sqrt(c.z) };
}

// Converts the red, green and blue channels of `c`.
//
// ```jai
// #import "Math";
// linear := srgb_to_linear(xyz(0.5, 0, 1));
// assert(abs(linear.x - 0.214) < 0.001 && linear.y == 0);
// ```
srgb_to_linear :: inline (c: Vector3) -> Vector3 {
    return .{ srgb_to_linear(c.x), srgb_to_linear(c.y), srgb_to_linear(c.z) };
}

// Converts the red, green and blue channels of `c`.
//
// ```jai
// #import "Math";
// encoded := linear_to_srgb(xyz(0.214, 0, 1));
// assert(abs(encoded.x - 0.5) < 0.001 && encoded.y == 0);
// ```
linear_to_srgb :: inline (c: Vector3) -> Vector3 {
    return .{ linear_to_srgb(c.x), linear_to_srgb(c.y), linear_to_srgb(c.z) };
}

// Converts the red, green and blue channels of `c`.
//
// ```jai
// #import "Math";
// linear := srgb_to_linear_fast(xyz(0.5, 0, 1));
// assert(abs(linear.x - 0.214) < 0.002);
// ```
srgb_to_linear_fast :: inline (c: Vector3) -> Vector3 {
    return .{ srgb_to_linear_fast(c.x), srgb_to_linear_fast(c.y), srgb_to_linear_fast(c.z) };
}

// Converts the red, green and blue channels of `c`.
//
// ```jai
// #import "Math";
// encoded := linear_to_srgb_fast(xyz(0.214, 0, 1));
// assert(abs(encoded.x - 0.5) < 0.003);
// ```
linear_to_srgb_fast :: inline (c: Vector3) -> Vector3 {
    return .{ linear_to_srgb_fast(c.x), linear_to_srgb_fast(c.y), linear_to_srgb_fast(c.z) };
}

// Squares the red, green and blue channels and leaves alpha as it is.
//
// ```jai
// #import "Math";
// assert(srgb_to_linear_pow2(xyzw(0.5, 1, 0, 0.3)) == xyzw(0.25, 1, 0, 0.3));
// ```
srgb_to_linear_pow2 :: inline (c: Vector4) -> Vector4 {
    return .{ c.x * c.x, c.y * c.y, c.z * c.z, c.w };
}

// Takes the square root of the red, green and blue channels and leaves alpha as it is.
//
// ```jai
// #import "Math";
// assert(linear_to_srgb_sqrt(xyzw(0.25, 1, 0, 0.3)) == xyzw(0.5, 1, 0, 0.3));
// ```
linear_to_srgb_sqrt :: inline (c: Vector4) -> Vector4 {
    return .{ sqrt(c.x), sqrt(c.y), sqrt(c.z), c.w };
}

// Converts the red, green and blue channels of `c` and leaves alpha as it is.
//
// ```jai
// #import "Math";
// linear := srgb_to_linear(xyzw(0.5, 0, 1, 0.3));
// assert(abs(linear.x - 0.214) < 0.001 && linear.w == 0.3);
// ```
srgb_to_linear :: inline (c: Vector4) -> Vector4 {
    return .{ srgb_to_linear(c.x), srgb_to_linear(c.y), srgb_to_linear(c.z), c.w };
}

// Converts the red, green and blue channels of `c` and leaves alpha as it is.
//
// ```jai
// #import "Math";
// encoded := linear_to_srgb(xyzw(0.214, 0, 1, 0.3));
// assert(abs(encoded.x - 0.5) < 0.001 && encoded.w == 0.3);
// ```
linear_to_srgb :: inline (c: Vector4) -> Vector4 {
    return .{ linear_to_srgb(c.x), linear_to_srgb(c.y), linear_to_srgb(c.z), c.w };
}

// Converts the red, green and blue channels of `c` and leaves alpha as it is.
//
// ```jai
// #import "Math";
// linear := srgb_to_linear_fast(xyzw(0.5, 0, 1, 0.3));
// assert(abs(linear.x - 0.214) < 0.002 && linear.w == 0.3);
// ```
srgb_to_linear_fast :: inline (c: Vector4) -> Vector4 {
    return .{ srgb_to_linear_fast(c.x), srgb_to_linear_fast(c.y), srgb_to_linear_fast(c.z), c.w };
}

// Converts the red, green and blue channels of `c` and leaves alpha as it is.
//
// ```jai
// #import "Math";
// encoded := linear_to_srgb_fast(xyzw(0.214, 0, 1, 0.3));
// assert(abs(encoded.x - 0.5) < 0.003 && encoded.w == 0.3);
// ```
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
