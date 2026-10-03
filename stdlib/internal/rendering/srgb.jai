srgb_to_linear :: inline (x: float) -> float {
    if x <= 0.04045 return x / 12.92;
    return pow((x + 0.055) / 1.055, 2.4);
}

linear_to_srgb :: (x: float) -> float {
    if x <= 0.0031308 return x * 12.92;
    return 1.055 * pow(x, 1.0 / 2.4) - 0.055;
}

// The fast path uses a piecewise linear interpolant of the exact transfer curve.
// Sampling in encoded space concentrates linear-domain samples near black.
srgb_to_linear_fast :: inline (x: float) -> float {
    if x <= 0.04045 return x / 12.92;
    if x >= 1 return srgb_to_linear(x);

    index := cast(int) (x * 64);
    lo := cast(float) index / 64;
    a := srgb_samples[index];
    b := srgb_samples[index + 1];

    return a + (b - a) * (x - lo) * 64;
}

linear_to_srgb_fast :: inline (x: float) -> float {
    if x <= 0.0031308 return x * 12.92;
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

srgb_to_linear_pow2 :: inline (c: Vector3) -> Vector3 {
    return .{ c.x * c.x, c.y * c.y, c.z * c.z };
}

linear_to_srgb_sqrt :: inline (c: Vector3) -> Vector3 {
    return .{ sqrt(c.x), sqrt(c.y), sqrt(c.z) };
}

srgb_to_linear :: inline (c: Vector3) -> Vector3 {
    return .{ srgb_to_linear(c.x), srgb_to_linear(c.y), srgb_to_linear(c.z) };
}

linear_to_srgb :: inline (c: Vector3) -> Vector3 {
    return .{ linear_to_srgb(c.x), linear_to_srgb(c.y), linear_to_srgb(c.z) };
}

srgb_to_linear_fast :: inline (c: Vector3) -> Vector3 {
    return .{ srgb_to_linear_fast(c.x), srgb_to_linear_fast(c.y), srgb_to_linear_fast(c.z) };
}

linear_to_srgb_fast :: inline (c: Vector3) -> Vector3 {
    return .{ linear_to_srgb_fast(c.x), linear_to_srgb_fast(c.y), linear_to_srgb_fast(c.z) };
}

srgb_to_linear_pow2 :: inline (c: Vector4) -> Vector4 {
    return .{ c.x * c.x, c.y * c.y, c.z * c.z, c.w };
}

linear_to_srgb_sqrt :: inline (c: Vector4) -> Vector4 {
    return .{ sqrt(c.x), sqrt(c.y), sqrt(c.z), c.w };
}

srgb_to_linear :: inline (c: Vector4) -> Vector4 {
    return .{ srgb_to_linear(c.x), srgb_to_linear(c.y), srgb_to_linear(c.z), c.w };
}

linear_to_srgb :: inline (c: Vector4) -> Vector4 {
    return .{ linear_to_srgb(c.x), linear_to_srgb(c.y), linear_to_srgb(c.z), c.w };
}

srgb_to_linear_fast :: inline (c: Vector4) -> Vector4 {
    return .{ srgb_to_linear_fast(c.x), srgb_to_linear_fast(c.y), srgb_to_linear_fast(c.z), c.w };
}

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
