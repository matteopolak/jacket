#import "Basic";
#import "Math";
H :: #import "Float16";
S :: #import "Sloppy_Math";

main :: () {
    assert(abs(cast(s8) -127) == 127 && abs(cast(s16) -32767) == 32767);
    assert(abs(cast(s32) -2147483647) == 2147483647 && abs(cast(s64) -9223372036854775807) == S64_MAX);
    x: s32 = 99;

    Clamp(*x, 0, 10);
    assert(x == 10);
    Clamp(*x, 20, 30);
    assert(x == 20);
    assert(frac(cast(float64) -1.25) == 0.75);
    assert(reduc(cast(float64) -0.01) == -0.0625);
    assert(fmod_cycling(cast(float64) -1.25, cast(float64) 1) == 0.75);
    assert(isinf(fmod_cycling(cast(float64) 1, cast(float64) 0)));
    assert(abs(sin_or_cos(cast(float64) 0, true)) < 0.000001);
    assert(sin_or_cos(cast(float64) 0, false) == 1 && cot(cast(float64) (PI64 / 4)) > 0.999999);
    assert(S.min_of_type(float64) == FLOAT64_MIN && S.max_of_type(float64) == FLOAT64_MAX);
    arr: [3] float = .[1, 2, 3];
    v := xyz(arr);

    assert(v.z == 3 && make_vector3(arr).y == 2);
    v2 := xy(1, 2);

    v = xyz(v2, 3);
    v4 := xyzw(v2, 3, 4);

    assert(xyzw(v, 4).w == 4);
    assert(make_vector2(3).y == 3 && make_vector4(3).w == 3 && xyz(3).z == 3);
    assert(abs(xy(-1, -2)).y == 2 && abs(v4).w == 4);
    assert(min(xy(1, 4), xy(2, 3)).y == 3 && max(v, xyz(4)).x == 4);
    assert(max(v4, xyzw(9)).w == 9 && min(v4, xyzw(0)).z == 0);
    assert(length_squared(xy(3, 4)) == 25 && length_squared(v4) == 30);
    assert(distance_squared(xy(0), xy(3, 4)) == 25 && distance(xyzw(0), v4) > 5);
    assert(multiply(xy(2), 3).x == 6 && multiply(xyz(2), 3).z == 6);
    fallback := xyz(9);
    empty := xyz(0);

    assert(normalize(*empty, 1.0, fallback) == 0 && empty.x == 9);
    assert(normalize(xyzw(0), fallback = xyzw(1)).w == 1 && unit_vector(xy(0)).x == 0);
    assert(saturate(xyzw(-1, 0.5, 2, 3)).w == 1 && saturate(xy(-1, 2)).y == 1);
    rotated2 := rotate(xy(1, 0), PI / 2);

    assert(abs(rotated2.y - 1) < 0.000001);
    rotate(*rotated2, -PI / 2);
    assert(abs(rotated2.x - 1) < 0.000001);
    assert(lerp(xy(0), xy(2), 0.25).x == 0.5 && lerp(xyzw(0), xyzw(2), 0.25).w == 0.5);
    m2 := Matrix2.{ 1, 2, 3, 4 };

    v2 = multiply(m2, xy(1, 2));
    assert(v2.x == 5 && v2.y == 11);
    m2 = multiply(m2, Matrix2_Identity);
    assert(m2._21 == 3);
    identity(*m2);
    assert(m2._11 == 1 && m2._12 == 0);
    m3 := Matrix3.{ 1, 2, 3, 4, 5, 6, 7, 8, 10 };
    copy3 := m3;

    transpose(*m3);
    assert(m3._12 == 4);
    transpose(*m3);
    assert(m3._12 == 2);
    assert(norm_inf(m3) == 25 && norm_one(m3) == 19);
    reflector := make_reflector(xyz(1, 2, 3));
    reflection := Matrix3_Identity;

    reflect_cols(*reflection, reflector);
    product3 := multiply(reflection, transpose(reflection));
    for i: 0..8 assert(abs(product3.floats[i] - Matrix3_Identity.floats[i]) < 0.000001);

    reflection = Matrix3_Identity;
    reflect_rows(*reflection, reflector);
    product3 = multiply(reflection, transpose(reflection));
    for i: 0..8 assert(abs(product3.floats[i] - Matrix3_Identity.floats[i]) < 0.000001);

    m3 = scale(Matrix3_Identity, xyz(2, 3, 4));
    scale(*m3, 2.0);
    assert(m3._33 == 8);
    scale(*m3, xyz(1, 2, 3));
    assert(m3._22 == 12);
    m3 = scale(m3, 0.5);
    assert(m3._33 == 12);
    assert(lerp(Matrix3_Identity, m3, 0.5)._33 == 6.5);
    ok3, inv3 := inverse(Matrix3.{ 1, 0, 0, 0, 1, 0, 0, 0, -0.001 });

    assert(ok3 && abs(inv3._33 + 1000) < 0.001);
    a4 := make_matrix4(copy3);
    a43 := Matrix4x3_Identity;
    a43._14 = 2;
    assert(make_matrix3(a4)._33 == 10 && transform_vector(a43, xyz(1, 2, 3)).x == 1);
    assert(multiply(a43, xyzw(1, 2, 3, 1)).x == 3);
    combined := multiply(a43, a43);

    assert(combined._14 == 4);
    a4 = multiply(Matrix4_Identity, Matrix3_Identity);
    assert(a4._44 == 1);
    a4 = multiply(Matrix4_Identity, a43);
    assert(a4._14 == 2);
    a4 = multiply(a43, Matrix4_Identity);
    assert(a4._14 == 2);
    assert(row(a4, 0).w == 2 && column(a4, 3).x == 2);
    transpose(*a4);
    assert(a4._41 == 2);
    transpose(*a4);
    translate(*a4, xyz(1, 2, 3));
    assert(a4._14 == 3);
    a4 = translate(a4, xyz(1));
    assert(a4._24 == 3);
    permutation := Matrix4.{ 0, 1, 0, 0, 1, 0, 0, 0, 0, 0, 0, 1, 0, 0, 1, 0 };
    ok4, inv4 := inverse_gaussian_elimination(permutation);

    assert(ok4 && inv4._12 == 1 && inv4._34 == 1);
    tiny4 := Matrix4_Identity;
    tiny4._44 = 1e-20;
    ok4, inv4 = inverse(tiny4);
    assert(ok4 && abs(inv4._44 * 1e-20 - 1) < 0.000001);
    assert(adjugate(Matrix4_Identity)._44 == 1 && adjugate_lengyel(Matrix4_Identity)._11 == 1);
    rows := make_matrix_from_rows(xyz(1, 2, 3), xyz(4, 5, 6), xyz(7, 8, 9), Matrix3);
    cols := make_matrix_from_columns(xyz(1, 2, 3), xyz(4, 5, 6), xyz(7, 8, 9));

    assert(rows._12 == 2 && cols._12 == 4);
    look := make_look_at_matrix(xyz(1, 0, 0), xyz(2, 0, 0), xyz(0, 0, 1));

    assert(distance(transform_point(look, xyz(1, 0, 0)), xyz(0)) < 0.00001);
    assert(transform_point(look, xyz(2, 0, 0)).x == 1);
    look = make_look_at_matrix(xyz(1, 0, 0), xyz(2, 0, 0), xyz(0, 0, 1), false);
    assert(transform_point(look, xyz(2, 0, 0)).z == -1);
    ortho := orthographic_projection_matrix(-2, 2, -3, 3, 1, 11, true);
    clip := multiply(ortho, xyzw(0, 0, -11, 1));

    assert(abs(clip.z - 1) < 0.00001);
    q: Quaternion;
    q.x = 0;
    q.y = 0;
    q.z = 0;
    q.w = 0;
    assert(normalize_or_identity(*q) == 0 && q.w == 1);
    end: Quaternion;

    set_from_axis_and_angle(*end, 0, 0, 1, PI / 2);
    mid := slerp(q, end, 0.5);

    assert(abs(length(mid) - 1) < 0.000001);
    mid = nlerp(q, end, 0.5);
    assert(abs(length(mid) - 1) < 0.000001);
    ax, ay, az, angle := get_axis_and_angle(end);

    assert(abs(ax) < cast(float64) 0.000001 && abs(ay) < cast(float64) 0.000001 && abs(az - 1) < cast(float64) 0.000001 && abs(angle - cast(float64) (PI / 2)) < cast(float64) 0.000001);
    assert(conjugate(end).z == -end.z);
    v = xyz(1, 0, 0);
    rotate(*v, end);
    assert(abs(v.y - 1) < 0.000001);
    rotated_matrix := rotate(Matrix3_Identity, end);

    assert(abs(rotated_matrix._21 - 1) < 0.000001);
    full_rotation := rotate(Matrix4_Identity, end);
    rigid := multiply(make_translation_matrix4(xyz(1, 2, 3)), full_rotation);
    inverse_rigid := isometry_inverse(rigid);
    product4 := multiply(rigid, inverse_rigid);

    for i: 0..15 assert(abs(product4.floats[i] - Matrix4_Identity.floats[i]) < 0.000001);

    y, z := make_an_orthonormal_basis(xyz(1, 0, 0));

    assert(abs(dot(y, z)) < 0.000001 && abs(length(y) - 1) < 0.000001);
    y, z = make_an_orthonormal_basis(xyz(1, 0, 0), xyz(1, 0, 0));
    assert(length(y) > 0.99 && length(z) > 0.99);
    p := make_plane(xyz(0, 0, 2), -4);
    p = normalize(p);
    assert(p.c == 1 && p.d == -2);
    assert(get_normal(p).z == 1);
    p = plane_from_point_and_normal(xyz(0, 0, 2), xyz(0, 0, 1));
    assert(plane_dot(p, xyz(0, 0, 2)) == 0);
    p = make_plane(xyzw(1, 2, 3, 4));
    assert(p.vector4.w == 4 && p.normal.z == 3);
    p = make_plane(1, 2, 3, 4);
    assert(p.b == 2);
    half := H.vector3_float16(xyz(1, 2, 3));

    v = H.vector3_float32(half);
    assert(v.z == 3);
    half2 := H.vector2_float16(xy(1, 2));

    assert(H.vector2_float32(half2).y == 2);
    half4 := H.vector4_float16(xyzw(1, 2, 3, 4));

    assert(cast(u16) half4.w == 0x4400);
    half_matrix := H.matrix3_float16(copy3);
    back := H.matrix3_float32(half_matrix);

    for i: 0..8 assert(back.floats[i] == copy3.floats[i]);

    print("algebra overloads ok\n");
}
