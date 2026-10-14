// Geometry, row-major matrix application, and IEEE half conversion vectors.
#import "Basic";
Math :: #import "Math";
Float16 :: #import "Float16";

main :: () {
    v := Math.xyz(2.0, -3.0, 6.0);

    assert(v.x == 2.0 && v.y == -3.0 && v.z == 6.0);
    assert(Math.dot_product(v, Math.xyz(1.0, 2.0, 0.5)) == -1.0);
    assert(Math.frac(-1.25) == 0.75);
    assert(Math.fmod_cycling(-1.25, 1.0) == 0.75);
    assert(Math.saturate(-0.25) == 0.0 && Math.saturate(1.25) == 1.0);
    assert(Math.is_finite(0.0) && Math.isinf(Math.FLOAT32_INFINITY));
    assert(Math.isnan(Math.FLOAT32_NAN) && !Math.is_finite(Math.FLOAT32_NAN));

    matrix := Math.Matrix4x3_Identity;
    matrix._14 = 5.0;
    point := Math.transform_point(matrix, Math.xyz(1.0, 2.0, 3.0));

    assert(point.x == 6.0 && point.y == 2.0 && point.z == 3.0);
    invertible, inverse := Math.inverse(Math.Matrix4_Identity);

    assert(invertible && inverse._11 == 1.0 && inverse._44 == 1.0);
    singular: Math.Matrix4 = .{};

    invertible, inverse = Math.inverse(singular);
    assert(!invertible);
    assert(type_info(Math.Vector3).name == "Vector3");
    assert(type_info(Float16.Vector3_Float16).name == "Vector3_Float16");

    one := Float16.float32_to_float16(1.0);
    negative_zero := Float16.float32_to_float16(-0.0);
    largest_finite := Float16.float32_to_float16(65504.0);
    overflow := Float16.float32_to_float16(70000.0);
    tie_even := Float16.float32_to_float16(cast(float32) 1.00048828125);

    assert(cast(u16) one == 0x3c00);
    assert(cast(u16) negative_zero == 0x8000);
    assert(cast(u16) largest_finite == 0x7bff);
    assert(cast(u16) overflow == 0x7c00);
    assert(cast(u16) tie_even == 0x3c00);
    assert(Float16.float16_to_float32(one) == 1.0);

    half_matrix: Float16.Matrix3_Float16;
    half_matrix._12 = Float16.float32_to_float16(2.0);
    assert(half_matrix.floats[1] == half_matrix._12);
    print("ok\n");
}
