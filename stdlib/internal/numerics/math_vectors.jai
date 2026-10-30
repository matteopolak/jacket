// The dot product of `a` and `b`.
//
// - a: the first vector.
// - b: the second vector.
dot_product :: (a: Vector2, b: Vector2) -> float {
    return a.x * b.x + a.y * b.y;
} @UseResult

// The squared length of `a`. Skips the square root.
//
// - a: the vector to measure.
length_squared :: inline (a: Vector2) -> float {
    return dot_product(a, a);
}

// The length of `a`. Computed in `float64` and scaled so huge or tiny components don't overflow. A
// NaN component gives NaN.
//
// - a: the vector to measure.
length :: (a: Vector2) -> float {
    return cast(float) length_wide(a);
} @UseResult

abs :: inline (using v: Vector2) -> Vector2 {
    return .{ abs(v.x), abs(v.y) };
} @UseResult

saturate :: (a: Vector2) -> Vector2 {
    return .{ saturate(a.x), saturate(a.y) };
}

lerp :: (a: Vector2, b: Vector2, t: float) -> Vector2 {
    return .{ lerp(a.x, b.x, t), lerp(a.y, b.y, t) };
} @UseResult

// The smaller of each pair of components.
//
// - v1: the first vector.
// - v2: the second vector.
min :: inline (v1: Vector2, v2: Vector2) -> Vector2 {
    return .{ min(v1.x, v2.x), min(v1.y, v2.y) };
} @UseResult @ReturnsSmaller

// The larger of each pair of components.
//
// - v1: the first vector.
// - v2: the second vector.
max :: inline (v1: Vector2, v2: Vector2) -> Vector2 {
    return .{ max(v1.x, v2.x), max(v1.y, v2.y) };
} @UseResult @ReturnsLarger

operator == :: (a: Vector2, b: Vector2) -> bool {
    return a.x == b.x && a.y == b.y;
}

operator + :: (a: Vector2, b: Vector2) -> Vector2 {
    return .{ a.x + b.x, a.y + b.y };
}

operator - :: (a: Vector2, b: Vector2) -> Vector2 {
    return .{ a.x - b.x, a.y - b.y };
}

operator * :: (a: Vector2, k: float) -> Vector2 #symmetric {
    return .{ a.x * k, a.y * k };
}

operator / :: (a: Vector2, k: float) -> Vector2 {
    return .{ a.x / k, a.y / k };
}

operator * :: (a: Vector2, b: Vector2) -> Vector2 {
    return .{ a.x * b.x, a.y * b.y };
}

operator / :: (a: Vector2, b: Vector2) -> Vector2 {
    return .{ a.x / b.x, a.y / b.y };
}

operator / :: (k: float, a: Vector2) -> Vector2 {
    return .{ k / a.x, k / a.y };
}

operator - :: (a: Vector2) -> Vector2 {
    return .{ -a.x, -a.y };
}

// A vector with both components set to `x`.
//
// - x: the value for each component.
make_vector2 :: (x: float) -> Vector2 {
    return .{ x, x };
}

// Short form of `Vector2.{ x, y }`.
//
// - x: the x component.
// - y: the y component.
xy :: (x: float, y: float) -> Vector2 {
    return .{ x, y };
}

// A vector with both components set to `x`.
//
// - x: the value for each component.
xy :: (x: float) -> Vector2 {
    return make_vector2(x);
}

// Scales every component of `v` by `f`.
//
// - v: the vector to scale.
// - f: the scale factor.
multiply :: (v: Vector2, f: float) -> Vector2 {
    return v * f;
}

// The squared distance between two points. Skips the square root.
//
// - a: the first point.
// - b: the second point.
distance_squared :: inline (a: Vector2, b: Vector2) -> float {
    return length_squared(a - b);
}

// `v` scaled to length 1. A zero vector is returned unchanged.
//
// - v: the vector to scale.
unit_vector :: (v: Vector2) -> Vector2 {
    return unit_vector_wide(v);
} @UseResult

// The distance between two points.
//
// - a: the first point.
// - b: the second point.
distance :: (a: Vector2, b: Vector2) -> float {
    return length(a - b);
}

// The dot product of `a` and `b`.
//
// - a: the first vector.
// - b: the second vector.
dot_product :: (a: Vector3, b: Vector3) -> float {
    return a.x * b.x + a.y * b.y + a.z * b.z;
} @UseResult

// The squared length of `a`. Skips the square root.
//
// - a: the vector to measure.
length_squared :: inline (a: Vector3) -> float {
    return dot_product(a, a);
}

// The length of `a`. Computed in `float64` and scaled so huge or tiny components don't overflow. A
// NaN component gives NaN.
//
// - a: the vector to measure.
length :: (a: Vector3) -> float {
    return cast(float) length_wide(a);
} @UseResult

abs :: inline (using v: Vector3) -> Vector3 {
    return .{ abs(v.x), abs(v.y), abs(v.z) };
} @UseResult

saturate :: (a: Vector3) -> Vector3 {
    return .{ saturate(a.x), saturate(a.y), saturate(a.z) };
}

lerp :: (a: Vector3, b: Vector3, t: float) -> Vector3 {
    return .{ lerp(a.x, b.x, t), lerp(a.y, b.y, t), lerp(a.z, b.z, t) };
} @UseResult

// The smaller of each pair of components.
//
// - v1: the first vector.
// - v2: the second vector.
min :: inline (v1: Vector3, v2: Vector3) -> Vector3 {
    return .{ min(v1.x, v2.x), min(v1.y, v2.y), min(v1.z, v2.z) };
} @UseResult @ReturnsSmaller

// The larger of each pair of components.
//
// - v1: the first vector.
// - v2: the second vector.
max :: inline (v1: Vector3, v2: Vector3) -> Vector3 {
    return .{ max(v1.x, v2.x), max(v1.y, v2.y), max(v1.z, v2.z) };
} @UseResult @ReturnsLarger

operator == :: (a: Vector3, b: Vector3) -> bool {
    return a.x == b.x && a.y == b.y && a.z == b.z;
}

operator + :: (a: Vector3, b: Vector3) -> Vector3 {
    return .{ a.x + b.x, a.y + b.y, a.z + b.z };
}

operator - :: (a: Vector3, b: Vector3) -> Vector3 {
    return .{ a.x - b.x, a.y - b.y, a.z - b.z };
}

operator * :: (a: Vector3, b: float) -> Vector3 {
    return .{ a.x * b, a.y * b, a.z * b };
}

operator / :: (a: Vector3, b: float) -> Vector3 {
    return .{ a.x / b, a.y / b, a.z / b };
}

operator * :: (a: Vector3, b: Vector3) -> Vector3 {
    return .{ a.x * b.x, a.y * b.y, a.z * b.z };
}

operator / :: (a: Vector3, b: Vector3) -> Vector3 {
    return .{ a.x / b.x, a.y / b.y, a.z / b.z };
}

operator / :: (k: float, a: Vector3) -> Vector3 {
    return .{ k / a.x, k / a.y, k / a.z };
}

operator - :: (a: Vector3) -> Vector3 {
    return .{ -a.x, -a.y, -a.z };
}

operator * :: (b: float, a: Vector3) -> Vector3 {
    return a * b;
}

// A float64 scale factor is narrowed to float32.
operator * :: (a: Vector3, k: float64) -> Vector3 {
    return a * cast(float) k;
}

operator * :: (k: float64, a: Vector3) -> Vector3 {
    return a * cast(float) k;
}

// A vector with all three components set to `x`.
//
// - x: the value for each component.
make_vector3 :: (x: float) -> Vector3 {
    return .{ x, x, x };
}

// Short form of `Vector3.{ x, y, z }`.
//
// - x: the x component.
// - y: the y component.
// - z: the z component.
xyz :: (x: float, y: float, z: float) -> Vector3 {
    return .{ x, y, z };
}

// A vector with all three components set to `x`.
//
// - x: the value for each component.
xyz :: (x: float) -> Vector3 {
    return make_vector3(x);
}

// Scales every component of `v` by `f`.
//
// - v: the vector to scale.
// - f: the scale factor.
multiply :: (v: Vector3, f: float) -> Vector3 {
    return v * f;
}

// The squared distance between two points. Skips the square root.
//
// - a: the first point.
// - b: the second point.
distance_squared :: inline (a: Vector3, b: Vector3) -> float {
    return length_squared(a - b);
}

// `v` scaled to length 1. A zero vector is returned unchanged.
//
// - v: the vector to scale.
unit_vector :: (v: Vector3) -> Vector3 {
    return unit_vector_wide(v);
} @UseResult

// The distance between two points.
//
// - a: the first point.
// - b: the second point.
distance :: (a: Vector3, b: Vector3) -> float {
    return length(a - b);
}

// The dot product of `a` and `b`.
//
// - a: the first vector.
// - b: the second vector.
dot_product :: (a: Vector4, b: Vector4) -> float {
    return a.x * b.x + a.y * b.y + a.z * b.z + a.w * b.w;
} @UseResult

// The squared length of `a`. Skips the square root.
//
// - a: the vector to measure.
length_squared :: inline (a: Vector4) -> float {
    return dot_product(a, a);
}

// The length of `a`. Computed in `float64` and scaled so huge or tiny components don't overflow. A
// NaN component gives NaN.
//
// - a: the vector to measure.
length :: (a: Vector4) -> float {
    return cast(float) length_wide(a);
} @UseResult

abs :: inline (using v: Vector4) -> Vector4 {
    return .{ abs(v.x), abs(v.y), abs(v.z), abs(v.w) };
} @UseResult

saturate :: (a: Vector4) -> Vector4 {
    return .{ saturate(a.x), saturate(a.y), saturate(a.z), saturate(a.w) };
}

lerp :: (a: Vector4, b: Vector4, t: float) -> Vector4 {
    return .{ lerp(a.x, b.x, t), lerp(a.y, b.y, t), lerp(a.z, b.z, t), lerp(a.w, b.w, t) };
} @UseResult

// The smaller of each pair of components.
//
// - v1: the first vector.
// - v2: the second vector.
min :: inline (v1: Vector4, v2: Vector4) -> Vector4 {
    return .{ min(v1.x, v2.x), min(v1.y, v2.y), min(v1.z, v2.z), min(v1.w, v2.w) };
} @UseResult @ReturnsSmaller

// The larger of each pair of components.
//
// - v1: the first vector.
// - v2: the second vector.
max :: inline (v1: Vector4, v2: Vector4) -> Vector4 {
    return .{ max(v1.x, v2.x), max(v1.y, v2.y), max(v1.z, v2.z), max(v1.w, v2.w) };
} @UseResult @ReturnsLarger

operator == :: (a: Vector4, b: Vector4) -> bool {
    return a.x == b.x && a.y == b.y && a.z == b.z && a.w == b.w;
}

operator + :: (a: Vector4, b: Vector4) -> Vector4 {
    return .{ a.x + b.x, a.y + b.y, a.z + b.z, a.w + b.w };
}

operator - :: (a: Vector4, b: Vector4) -> Vector4 {
    return .{ a.x - b.x, a.y - b.y, a.z - b.z, a.w - b.w };
}

operator * :: (a: Vector4, b: float) -> Vector4 {
    return .{ a.x * b, a.y * b, a.z * b, a.w * b };
}

operator / :: (a: Vector4, b: float) -> Vector4 {
    return .{ a.x / b, a.y / b, a.z / b, a.w / b };
}

operator * :: (a: Vector4, b: Vector4) -> Vector4 {
    return .{ a.x * b.x, a.y * b.y, a.z * b.z, a.w * b.w };
}

operator / :: (a: Vector4, b: Vector4) -> Vector4 {
    return .{ a.x / b.x, a.y / b.y, a.z / b.z, a.w / b.w };
}

operator / :: (k: float, a: Vector4) -> Vector4 {
    return .{ k / a.x, k / a.y, k / a.z, k / a.w };
}

operator - :: (a: Vector4) -> Vector4 {
    return .{ -a.x, -a.y, -a.z, -a.w };
}

operator * :: (b: float, a: Vector4) -> Vector4 {
    return a * b;
}

// A vector with all four components set to `x`.
//
// - x: the value for each component.
make_vector4 :: (x: float) -> Vector4 {
    return .{ x, x, x, x };
}

// Short form of `Vector4.{ x, y, z, w }`.
//
// - x: the x component.
// - y: the y component.
// - z: the z component.
// - w: the w component.
xyzw :: (x: float, y: float, z: float, w: float) -> Vector4 {
    return .{ x, y, z, w };
}

// A vector with all four components set to `x`.
//
// - x: the value for each component.
xyzw :: (x: float) -> Vector4 {
    return make_vector4(x);
}

// Scales every component of `v` by `f`.
//
// - v: the vector to scale.
// - f: the scale factor.
multiply :: (v: Vector4, f: float) -> Vector4 {
    return v * f;
}

// The squared distance between two points. Skips the square root.
//
// - a: the first point.
// - b: the second point.
distance_squared :: inline (a: Vector4, b: Vector4) -> float {
    return length_squared(a - b);
}

// `v` scaled to length 1. A zero vector is returned unchanged.
//
// - v: the vector to scale.
unit_vector :: (v: Vector4) -> Vector4 {
    return unit_vector_wide(v);
} @UseResult

// The distance between two points.
//
// - a: the first point.
// - b: the second point.
distance :: (a: Vector4, b: Vector4) -> float {
    return length(a - b);
}

dot :: dot_product;
rgb :: xyz;
rgba :: xyzw;
MyVectors :: .[Vector2, Vector3, Vector4, Quaternion];

// Scales `*v` to length 1 in place and returns its old length. A length at or below `epsilon` is
// treated as zero: `*v` becomes `fallback` and that length is still returned.
//
// - v: the vector to normalize.
// - epsilon: lengths up to this value count as zero.
// - fallback: what `*v` is set to when the length counts as zero.
normalize :: (v: *$T/MyVectors, epsilon := 0.0, fallback := T.{}) -> float {
    wide := length_wide(v.*);
    n := cast(float) wide;

    if n <= epsilon v.* = fallback;
    else {
        for *x: v.component x.* = cast(float) (cast(float64) x.* / wide);
    }

    return n;
}

// Returns a copy of `v` scaled to length 1. A length at or below `epsilon` is treated as zero and
// `fallback` is returned instead.
//
// - v: the vector to normalize.
// - epsilon: lengths up to this value count as zero.
// - fallback: what is returned when the length counts as zero.
normalize :: (v: $T/MyVectors, epsilon := 0.0, fallback := T.{}) -> T {
    r := v;
    normalize(*r, epsilon, fallback);
    return r;
} @UseResult

// A vector from an array of x, y, z.
//
// - c: the three components, in x, y, z order.
make_vector3 :: (c: [3] float) -> Vector3 {
    return .{ c[0], c[1], c[2] };
}

// Extends a `Vector2` with a z component.
//
// - v: supplies x and y.
// - z: the z component.
make_vector3 :: (using v: Vector2, z: float) -> Vector3 {
    return .{ x, y, z };
}

// A vector from an array of x, y, z.
//
// - c: the three components, in x, y, z order.
xyz :: (c: [3] float) -> Vector3 {
    return make_vector3(c);
}

// Extends a `Vector2` with a z component.
//
// - v: supplies x and y.
// - z: the z component.
xyz :: (using v: Vector2, z: float) -> Vector3 {
    return make_vector3(v, z);
}

// Extends a `Vector2` with z and w components.
//
// - v: supplies x and y.
// - z: the z component.
// - w: the w component.
make_vector4 :: (using v: Vector2, z: float, w: float) -> Vector4 {
    return .{ x, y, z, w };
}

// Extends a `Vector3` with a w component.
//
// - v: supplies x, y and z.
// - w: the w component.
make_vector4 :: (using v: Vector3, w: float) -> Vector4 {
    return .{ x, y, z, w };
}

// Extends a `Vector2` with z and w components.
//
// - v: supplies x and y.
// - z: the z component.
// - w: the w component.
xyzw :: (using v: Vector2, z: float, w: float) -> Vector4 {
    return make_vector4(v, z, w);
}

// Extends a `Vector3` with a w component.
//
// - v: supplies x, y and z.
// - w: the w component.
xyzw :: (using v: Vector3, w: float) -> Vector4 {
    return make_vector4(v, w);
}

cross :: cross_product;

// The cross product of `a` and `b`, perpendicular to both. Also available as `cross`.
//
// - a: the left vector.
// - b: the right vector.
cross_product :: (a: Vector3, b: Vector3) -> Vector3 {
    return .{ a.y * b.z - a.z * b.y, a.z * b.x - a.x * b.z, a.x * b.y - a.y * b.x };
} @UseResult

// Rotates `v` counterclockwise about the origin.
//
// - v: the vector to rotate.
// - theta: the angle in radians.
rotate :: (v: Vector2, theta: float) -> Vector2 {
    c := cos(theta);
    s := sin(theta);
    return .{ c * v.x - s * v.y, s * v.x + c * v.y };
}

// Rotates `*v` counterclockwise about the origin, in place.
//
// - v: the vector to rotate.
// - theta: the angle in radians.
rotate :: (v: *Vector2, theta: float) {
    v.* = rotate(v.*, theta);
}

#scope_module
// Preserve range and significand precision until the final public binary32
// result. Normalization must not divide by that rounded/overflowed result.
length_wide :: (v: $T/MyVectors) -> float64 {
    largest: float64 = 0;

    for x: v.component {
        if isnan(x) return FLOAT64_NAN;
        largest = max(largest, abs(cast(float64) x));
    }

    if largest == 0 || isinf(largest) return largest;

    sum: float64 = 0;

    for x: v.component {
        scaled := cast(float64) x / largest;
        sum += scaled * scaled;
    }

    return largest * sqrt(sum);
}

unit_vector_wide :: (v: $T/MyVectors) -> T {
    wide := length_wide(v);
    if wide == 0 return v;

    r := v;
    for *x: r.component x.* = cast(float) (cast(float64) x.* / wide);

    return r;
}

#scope_file
#import "Basic";
