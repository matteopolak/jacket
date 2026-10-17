dot_product :: (a: Vector2, b: Vector2) -> float {
    return a.x * b.x + a.y * b.y;
} @UseResult

length_squared :: inline (a: Vector2) -> float {
    return dot_product(a, a);
}

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

min :: inline (v1: Vector2, v2: Vector2) -> Vector2 {
    return .{ min(v1.x, v2.x), min(v1.y, v2.y) };
} @UseResult @Minimum

max :: inline (v1: Vector2, v2: Vector2) -> Vector2 {
    return .{ max(v1.x, v2.x), max(v1.y, v2.y) };
} @UseResult @Maximum

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

make_vector2 :: (x: float) -> Vector2 {
    return .{ x, x };
}

xy :: (x: float, y: float) -> Vector2 {
    return .{ x, y };
}

xy :: (x: float) -> Vector2 {
    return make_vector2(x);
}

multiply :: (v: Vector2, f: float) -> Vector2 {
    return v * f;
}

distance_squared :: inline (a: Vector2, b: Vector2) -> float {
    return length_squared(a - b);
}

unit_vector :: (v: Vector2) -> Vector2 {
    return unit_vector_wide(v);
} @UseResult

distance :: (a: Vector2, b: Vector2) -> float {
    return length(a - b);
}

dot_product :: (a: Vector3, b: Vector3) -> float {
    return a.x * b.x + a.y * b.y + a.z * b.z;
} @UseResult

length_squared :: inline (a: Vector3) -> float {
    return dot_product(a, a);
}

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

min :: inline (v1: Vector3, v2: Vector3) -> Vector3 {
    return .{ min(v1.x, v2.x), min(v1.y, v2.y), min(v1.z, v2.z) };
} @UseResult @Minimum

max :: inline (v1: Vector3, v2: Vector3) -> Vector3 {
    return .{ max(v1.x, v2.x), max(v1.y, v2.y), max(v1.z, v2.z) };
} @UseResult @Maximum

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

operator - :: (a: Vector3) -> Vector3 {
    return .{ -a.x, -a.y, -a.z };
}

operator * :: (b: float, a: Vector3) -> Vector3 {
    return a * b;
}

make_vector3 :: (x: float) -> Vector3 {
    return .{ x, x, x };
}

xyz :: (x: float, y: float, z: float) -> Vector3 {
    return .{ x, y, z };
}

xyz :: (x: float) -> Vector3 {
    return make_vector3(x);
}

multiply :: (v: Vector3, f: float) -> Vector3 {
    return v * f;
}

distance_squared :: inline (a: Vector3, b: Vector3) -> float {
    return length_squared(a - b);
}

unit_vector :: (v: Vector3) -> Vector3 {
    return unit_vector_wide(v);
} @UseResult

distance :: (a: Vector3, b: Vector3) -> float {
    return length(a - b);
}

dot_product :: (a: Vector4, b: Vector4) -> float {
    return a.x * b.x + a.y * b.y + a.z * b.z + a.w * b.w;
} @UseResult

length_squared :: inline (a: Vector4) -> float {
    return dot_product(a, a);
}

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

min :: inline (v1: Vector4, v2: Vector4) -> Vector4 {
    return .{ min(v1.x, v2.x), min(v1.y, v2.y), min(v1.z, v2.z), min(v1.w, v2.w) };
} @UseResult @Minimum

max :: inline (v1: Vector4, v2: Vector4) -> Vector4 {
    return .{ max(v1.x, v2.x), max(v1.y, v2.y), max(v1.z, v2.z), max(v1.w, v2.w) };
} @UseResult @Maximum

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

operator - :: (a: Vector4) -> Vector4 {
    return .{ -a.x, -a.y, -a.z, -a.w };
}

operator * :: (b: float, a: Vector4) -> Vector4 {
    return a * b;
}

make_vector4 :: (x: float) -> Vector4 {
    return .{ x, x, x, x };
}

xyzw :: (x: float, y: float, z: float, w: float) -> Vector4 {
    return .{ x, y, z, w };
}

xyzw :: (x: float) -> Vector4 {
    return make_vector4(x);
}

distance :: (a: Vector4, b: Vector4) -> float {
    return length(a - b);
}

dot :: dot_product;
rgb :: xyz;
rgba :: xyzw;
MyVectors :: .[Vector2, Vector3, Vector4, Quaternion];

normalize :: (v: *$T/MyVectors, epsilon := 0.0, fallback := T.{}) -> float {
    wide := length_wide(v.*);
    n := cast(float) wide;

    if n <= epsilon v.* = fallback;
    else {
        for *x: v.component x.* = cast(float) (cast(float64) x.* / wide);
    }

    return n;
}

normalize :: (v: $T/MyVectors, epsilon := 0.0, fallback := T.{}) -> T {
    r := v;
    normalize(*r, epsilon, fallback);
    return r;
} @UseResult

make_vector3 :: (c: [3] float) -> Vector3 {
    return .{ c[0], c[1], c[2] };
}

make_vector3 :: (using v: Vector2, z: float) -> Vector3 {
    return .{ x, y, z };
}

xyz :: (c: [3] float) -> Vector3 {
    return make_vector3(c);
}

xyz :: (using v: Vector2, z: float) -> Vector3 {
    return make_vector3(v, z);
}

make_vector4 :: (using v: Vector2, z: float, w: float) -> Vector4 {
    return .{ x, y, z, w };
}

make_vector4 :: (using v: Vector3, w: float) -> Vector4 {
    return .{ x, y, z, w };
}

xyzw :: (using v: Vector2, z: float, w: float) -> Vector4 {
    return make_vector4(v, z, w);
}

xyzw :: (using v: Vector3, w: float) -> Vector4 {
    return make_vector4(v, w);
}

cross :: cross_product;

cross_product :: (a: Vector3, b: Vector3) -> Vector3 {
    return .{ a.y * b.z - a.z * b.y, a.z * b.x - a.x * b.z, a.x * b.y - a.y * b.x };
} @UseResult

rotate :: (v: Vector2, theta: float) -> Vector2 {
    c := cos(theta);
    s := sin(theta);
    return .{ c * v.x - s * v.y, s * v.x + c * v.y };
}

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
