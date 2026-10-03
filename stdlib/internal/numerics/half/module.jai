#module_parameters(ASSERT_ON_OVERFLOW := false);
Float16 :: #type,isa u16;

Vector2_Float16 :: struct {
    x, y: Float16;
}

Vector3_Float16 :: struct {
    x, y, z: Float16;
}

Vector4_Float16 :: struct {
    x, y, z, w: Float16;
}

Matrix3_Float16 :: struct {
    _11, _12, _13, _21, _22, _23, _31, _32, _33: Float16;
    #overlay(_11) floats: [9] Float16;
}

vector2_float32 :: (v: Vector2_Float16) -> Vector2 {
    return .{ half_to_float_fast4(v.x), half_to_float_fast4(v.y) };
}

vector3_float32 :: (v: Vector3_Float16) -> Vector3 {
    return .{ half_to_float_fast4(v.x), half_to_float_fast4(v.y), half_to_float_fast4(v.z) };
}

vector2_float16 :: (v: Vector2) -> Vector2_Float16 {
    return .{ float_to_half_fast3(v.x), float_to_half_fast3(v.y) };
}

vector3_float16 :: (v: Vector3) -> Vector3_Float16 {
    return .{ float_to_half_fast3(v.x), float_to_half_fast3(v.y), float_to_half_fast3(v.z) };
}

vector4_float16 :: (v: Vector4) -> Vector4_Float16 {
    return .{ float_to_half_fast3(v.x), float_to_half_fast3(v.y), float_to_half_fast3(v.z), float_to_half_fast3(v.w) };
}

matrix3_float16 :: (s: Matrix3) -> Matrix3_Float16 {
    r: Matrix3_Float16;
    for i: 0..8 r.floats[i] = float_to_half_fast3(s.floats[i]);
    return r;
}

matrix3_float32 :: (s: Matrix3_Float16) -> Matrix3 {
    r: Matrix3;
    for i: 0..8 r.floats[i] = half_to_float_fast4(s.floats[i]);
    return r;
}

float32_to_float16 :: float_to_half_fast3;
float16_to_float32 :: half_to_float_fast4;

half_to_float_fast4 :: (h: Float16) -> float32 {
    return half_to_float(cast(u16) h);
}

float_to_half_fast3 :: (_f: float32) -> Float16 {
    b: Bits;
    b.f = _f;
    sign: u32 = (b.u >> 16) & 0x8000;
    magnitude := b.u & 0x7fffffff;

    if magnitude >= 0x7f800000 {
        if magnitude == 0x7f800000 return cast(Float16) (sign | 0x7c00);
        payload := (magnitude >> 13) & 0x3ff;
        payload |= 0x200;
        return cast(Float16) (sign | 0x7c00 | payload);
    }

    // Round the integer significand, including the implicit leading bit, exactly once.
    e := cast(s32) (magnitude >> 23) - 127;
    m := magnitude & 0x7fffff;

    if e > 15 {
        #if ASSERT_ON_OVERFLOW assert(false, "half float overflow");
        return cast(Float16) (sign | 0x7c00);
    }

    if e < -25 return cast(Float16) sign;

    shift: u32 = 13;

    if e < -14 {
        m |= 0x800000;
        shift = cast(u32) (-e - 1);
    }

    rounded := m >> shift;
    tail := m & ((cast(u32) 1 << shift) - 1);
    halfway := cast(u32) 1 << (shift - 1);

    if tail > halfway || (tail == halfway && (rounded & 1) != 0) rounded += 1;

    encoded: u32;
    if e < -14 encoded = rounded;
    else encoded = (cast(u32) (e + 15) << 10) + rounded;

    if encoded >= 0x7c00 {
        #if ASSERT_ON_OVERFLOW assert(false, "half float overflow");
        encoded = 0x7c00;
    }

    return cast(Float16) (sign | encoded);
}

#scope_file
#import "Math";
#import "Basic";

Bits :: union {
    f: float32;
    u: u32;
}
