// Math vectors, matrices, quaternions and planes at run time, checked with identities
// (M * inverse(M) = I, Q orthogonal, rotations preserve length, ...) and hand-computed values.
#import "Basic";
#import "Math";

EPS :: 0.0001;

close :: (a: float, b: float, eps: float = EPS) -> bool {
    return abs(a - b) <= eps * (1 + abs(b));
}

close :: (a: Vector2, b: Vector2, eps: float = EPS) -> bool {
    return close(a.x, b.x, eps) && close(a.y, b.y, eps);
}

close :: (a: Vector3, b: Vector3, eps: float = EPS) -> bool {
    return close(a.x, b.x, eps) && close(a.y, b.y, eps) && close(a.z, b.z, eps);
}

close :: (a: Vector4, b: Vector4, eps: float = EPS) -> bool {
    return close(a.x, b.x, eps) && close(a.y, b.y, eps) && close(a.z, b.z, eps) && close(a.w, b.w, eps);
}

close_floats :: (a: [] float, b: [] float, eps: float = EPS) -> bool {
    if a.count != b.count return false;
    for a if !close(it, b[it_index], eps) return false;
    return true;
}

close :: (a: Matrix3, b: Matrix3, eps: float = EPS) -> bool {
    return close_floats(a.floats, b.floats, eps);
}

close :: (a: Matrix4, b: Matrix4, eps: float = EPS) -> bool {
    return close_floats(a.floats, b.floats, eps);
}

// The same rotation: q and -q are equal rotations.
same_rotation :: (a: Quaternion, b: Quaternion) -> bool {
    return abs(abs(dot(a, b)) - 1) <= EPS;
}

main :: () {
    vectors();
    quaternions();
    small_matrices();
    matrix4_algebra();
    transforms();
    decompositions();
    planes();
    print("ok\n");
}

vectors :: () {
    // Constructors.
    assert(make_vector2(1, 2) == Vector2.{ 1, 2 } && make_vector2(3) == Vector2.{ 3, 3 } && xy(4) == Vector2.{ 4, 4 });
    assert(make_vector3(5) == Vector3.{ 5, 5, 5 } && xyz(1) == Vector3.{ 1, 1, 1 } && make_vector4(2) == Vector4.{ 2, 2, 2, 2 } && xyzw(7) == Vector4.{ 7, 7, 7, 7 });
    assert(make_vector3(float.[1, 2, 3]) == Vector3.{ 1, 2, 3 } && xyz(float.[4, 5, 6]) == Vector3.{ 4, 5, 6 });
    assert(make_vector3(Vector2.{ 1, 2 }, 3) == Vector3.{ 1, 2, 3 } && xyz(Vector2.{ 1, 2 }, 9) == Vector3.{ 1, 2, 9 });
    assert(make_vector4(Vector2.{ 1, 2 }, 3, 4) == Vector4.{ 1, 2, 3, 4 } && make_vector4(Vector3.{ 1, 2, 3 }, 4) == Vector4.{ 1, 2, 3, 4 });
    assert(xyzw(Vector2.{ 1, 2 }, 3, 4) == Vector4.{ 1, 2, 3, 4 } && rgba(Vector3.{ 0.1, 0.2, 0.3 }, 1).w == 1 && rgb(1, 0, 0) == Vector3.{ 1, 0, 0 });

    // Vector2 arithmetic and helpers.
    a := Vector2.{ 3, -4 };
    b := Vector2.{ 1, 2 };
    assert(a + b == Vector2.{ 4, -2 } && a - b == Vector2.{ 2, -6 } && a * b == Vector2.{ 3, -8 } && a / b == Vector2.{ 3, -2 });
    assert(-a == Vector2.{ -3, 4 } && a * 2 == Vector2.{ 6, -8 } && 2 * a == Vector2.{ 6, -8 } && a / 2 == Vector2.{ 1.5, -2 } && 12 / a == Vector2.{ 4, -3 });
    assert(dot(a, b) == -5 && length(a) == 5 && length_squared(a) == 25);
    assert(distance(a, b) == sqrt(cast(float) 40) && distance_squared(a, b) == 40 && multiply(b, 3) == Vector2.{ 3, 6 });
    assert(min(a, b) == Vector2.{ 1, -4 } && max(a, b) == Vector2.{ 3, 2 } && abs(a) == Vector2.{ 3, 4 } && saturate(a) == Vector2.{ 1, 0 });
    assert(lerp(a, b, 0.5) == Vector2.{ 2, -1 } && close(unit_vector(a), Vector2.{ 0.6, -0.8 }));
    assert(close(rotate(Vector2.{ 1, 0 }, PI / 2), Vector2.{ 0, 1 }) && close(rotate(Vector2.{ 1, 1 }, PI), Vector2.{ -1, -1 }));
    spun := Vector2.{ 0, 2 };
    rotate(*spun, -PI / 2);
    assert(close(spun, Vector2.{ 2, 0 }), "rotate in place: %", spun);
    i := Complex.{ 0, 1 };
    assert(i * i == Complex.{ -1, 0 }, "complex i^2");
    assert((Complex.{ 1, 2 }) * (Complex.{ 3, 4 }) == Complex.{ -5, 10 });

    // Vector3.
    p := Vector3.{ 1, 2, 2 };
    q := Vector3.{ -2, 0, 1 };
    assert(p + q == Vector3.{ -1, 2, 3 } && p - q == Vector3.{ 3, 2, 1 } && p * q == Vector3.{ -2, 0, 2 } && p / Vector3.{ 1, 2, 4 } == Vector3.{ 1, 1, 0.5 });
    assert(-p == Vector3.{ -1, -2, -2 } && p * 3 == Vector3.{ 3, 6, 6 } && 3 * p == p * 3 && p / 2 == Vector3.{ 0.5, 1, 1 } && 4 / p == Vector3.{ 4, 2, 2 });
    assert(p * cast(float64) 2 == Vector3.{ 2, 4, 4 } && cast(float64) 2 * p == Vector3.{ 2, 4, 4 });
    assert(dot(p, q) == 0 && length(p) == 3 && length_squared(p) == 9 && distance(p, p) == 0 && distance_squared(p, q) == 14 && close(distance(p, q), sqrt(cast(float) 14)));
    assert(multiply(p, -1) == -p && min(p, q) == Vector3.{ -2, 0, 1 } && max(p, q) == Vector3.{ 1, 2, 2 } && abs(q) == Vector3.{ 2, 0, 1 });
    assert(saturate(Vector3.{ -1, 0.5, 2 }) == Vector3.{ 0, 0.5, 1 } && lerp(p, q, 1) == q);
    assert(cross(Vector3.{ 1, 0, 0 }, Vector3.{ 0, 1, 0 }) == Vector3.{ 0, 0, 1 } && cross(p, p) == Vector3.{});
    c := cross(p, q);
    assert(dot(c, p) == 0 && dot(c, q) == 0, "cross is orthogonal to both");
    assert(close(unit_vector(p), Vector3.{ 1.0 / 3, 2.0 / 3, 2.0 / 3 }) && close(length(normalize(q)), 1));
    zero := Vector3.{};
    assert(normalize(zero) == Vector3.{} && normalize(zero, fallback = .{ z = 1 }) == Vector3.{ 0, 0, 1 }, "a zero vector normalizes to the fallback");
    scaled := Vector3.{ 0, 0, 5 };
    old_length := normalize(*scaled);
    assert(old_length == 5 && scaled == Vector3.{ 0, 0, 1 });
    tiny := Vector3.{ 0.001, 0, 0 };
    normalize(*tiny, epsilon = 0.01, fallback = .{ y = 1 });
    assert(tiny == Vector3.{ 0, 1, 0 }, "under epsilon uses the fallback");
    assert(p.xy == Vector2.{ 1, 2 } && p.yz == Vector2.{ 2, 2 } && p.component[2] == 2);

    // Vector4.
    v := Vector4.{ 1, 2, 3, 4 };
    w := Vector4.{ 4, 3, 2, 1 };
    assert(v + w == xyzw(5) && v - w == Vector4.{ -3, -1, 1, 3 } && v * w == Vector4.{ 4, 6, 6, 4 } && v / w == Vector4.{ 0.25, 2.0 / 3, 1.5, 4 });
    assert(-v == Vector4.{ -1, -2, -3, -4 } && v * 2 == 2 * v && v / 2 == Vector4.{ 0.5, 1, 1.5, 2 } && 12 / v == Vector4.{ 12, 6, 4, 3 });
    assert(dot(v, w) == 20 && length_squared(v) == 30 && close(length(v), sqrt(cast(float) 30)) && distance_squared(v, w) == 20 && close(distance(v, w), sqrt(cast(float) 20)));
    assert(min(v, w) == Vector4.{ 1, 2, 2, 1 } && max(v, w) == Vector4.{ 4, 3, 3, 4 } && abs(-v) == v && saturate(v * 0.25) == Vector4.{ 0.25, 0.5, 0.75, 1 });
    assert(lerp(v, w, 0.5) == xyzw(2.5) && v.xyz == Vector3.{ 1, 2, 3 } && v.zw == Vector2.{ 3, 4 } && v.yzw == Vector3.{ 2, 3, 4 });

    // Orthonormal bases from any axis.
    for axis: Vector3.[.{ 1, 0, 0 }, .{ 0, 0, 1 }, .{ 0.3, -0.8, 0.52 }, .{ 0, 1, 0 }, .{ 0, 0, -7 }] {
        x := normalize(axis);
        y, z := make_an_orthonormal_basis(axis);
        assert(close(length(y), 1) && close(length(z), 1) && abs(dot(x, y)) < EPS && abs(dot(x, z)) < EPS && abs(dot(y, z)) < EPS, "basis for %", axis);
        assert(close(cross(x, y), z), "right-handed basis for %", axis);
        y2, z2 := make_an_orthonormal_basis(axis, Vector3.{ 0, 1, 0 });
        assert(close(length(y2), 1) && abs(dot(x, y2)) < EPS && abs(dot(x, z2)) < EPS && close(cross(x, y2), z2), "basis with a hint for %", axis);
    }
}

quaternions :: () {
    identity: Quaternion;
    assert(identity == Quaternion_Identity && identity.w == 1 && length(identity) == 1);
    q: Quaternion;
    set_from_axis_and_angle(*q, 0, 0, 1, PI / 2);
    assert(close(q.z, sqrt(0.5)) && close(q.w, sqrt(0.5)));
    x_axis := Vector3.{ 1, 0, 0 };
    assert(close(rotate(x_axis, q), Vector3.{ 0, 1, 0 }), "90 degrees about z");
    moved := x_axis;
    rotate(*moved, q);
    rotate(*moved, q);
    assert(close(moved, Vector3.{ -1, 0, 0 }), "twice gives 180 degrees: %", moved);

    // Composition: q * r rotates by r first, then q.
    r: Quaternion;
    set_from_axis_and_angle(*r, Vector3.{ 1, 0, 0 }, PI / 2);
    composed := q * r;
    assert(close(rotate(Vector3.{ 0, 1, 0 }, composed), rotate(rotate(Vector3.{ 0, 1, 0 }, r), q)));
    assert(close(length(composed), 1) && close(length_squared(composed), 1));
    inverse_q := conjugate(q);
    assert(close(rotate(rotate(Vector3.{ 0.3, 0.4, 0.5 }, q), inverse_q), Vector3.{ 0.3, 0.4, 0.5 }), "conjugate undoes");
    assert(same_rotation(q * inverse_q, Quaternion_Identity));
    assert(-q == Quaternion.{ -q.x, -q.y, -q.z, -q.w } && q + q == q * 2 && q - q == Quaternion.{ 0, 0, 0, 0 } && 2 * q == q * 2);

    // Axis and angle round trip, including the identity.
    ax, ay, az, angle := get_axis_and_angle(q);
    assert(close(cast(float) az, 1) && close(cast(float) ax, 0) && close(cast(float) angle, PI / 2), "axis-angle: % % % %", ax, ay, az, angle);
    ax, ay, az, angle = get_axis_and_angle(Quaternion_Identity);
    assert(ax == 1 && ay == 0 && az == 0 && angle == 0, "the identity has a zero angle");

    // Interpolation.
    half := slerp(Quaternion_Identity, q, 0.5);
    _, _, hz, half_angle := get_axis_and_angle(half);
    assert(close(cast(float) half_angle, PI / 4) && close(cast(float) hz, 1));
    assert(same_rotation(slerp(Quaternion_Identity, q, 0), Quaternion_Identity) && same_rotation(slerp(Quaternion_Identity, q, 1), q));
    assert(same_rotation(slerp(Quaternion_Identity, -q, 0.5), half), "slerp takes the short way round");
    nearby: Quaternion;
    set_from_axis_and_angle(*nearby, 0, 0, 1, 0.01);
    assert(close(length(slerp(Quaternion_Identity, nearby, 0.3)), 1), "nearly equal inputs use nlerp");
    n := nlerp(Quaternion_Identity, q, 0.5);
    assert(close(length(n), 1) && same_rotation(n, half), "nlerp halfway equals slerp halfway by symmetry");
    flipped := lerp(Quaternion_Identity, -Quaternion_Identity, 0.5);
    assert(flipped == Quaternion_Identity, "lerp flips the target into the same hemisphere: %", flipped);

    // normalize_or_identity.
    degenerate := Quaternion.{ 0, 0, 0, 0 };
    assert(normalize_or_identity(*degenerate) == 0 && degenerate == Quaternion_Identity);
    stretched := Quaternion.{ 0, 0, 3, 4 };
    assert(normalize_or_identity(*stretched) == 5 && close(stretched.z, 0.6) && close(stretched.w, 0.8));
}

small_matrices :: () {
    // Matrix2.
    m2 := Matrix2.{ 1, 2, 3, 4 };
    assert(m2 * Vector2.{ 1, 1 } == Vector2.{ 3, 7 } && multiply(m2, Vector2.{ 1, 0 }) == Vector2.{ 1, 3 });
    assert(m2 * m2 == Matrix2.{ 7, 10, 15, 22 } && multiply(m2, Matrix2_Identity) == m2);
    assert(m2 + m2 == m2 * 2 && 2 * m2 == m2 * 2 && m2 - m2 == Matrix2.{} && !(m2 == Matrix2_Identity));
    id2: Matrix2;
    identity(*id2);
    assert(id2 == Matrix2_Identity && identity_of(Matrix2) == Matrix2_Identity);
    assert(m2.coef[1][0] == 3 && m2.v[1] == Vector2.{ 3, 4 } && m2.floats[3] == 4);

    // Matrix3.
    m3 := Matrix3.{ 2, 0, 1, 1, 3, 0, 0, 1, 4 };
    assert(m3 * Vector3.{ 1, 1, 1 } == Vector3.{ 3, 4, 5 } && transform_vector(m3, Vector3.{ 1, 0, 0 }) == Vector3.{ 2, 1, 0 });
    assert(m3 * Matrix3_Identity == m3 && Matrix3_Identity * m3 == m3);
    assert((m3 * m3).coef[0][0] == 4 && (m3 * m3)._13 == 6 && (m3 * m3)._33 == 16, "Matrix3 product: %", m3 * m3);
    assert(m3 + m3 == m3 * 2 && 3 * m3 == m3 * 3 && m3 - m3 == Matrix3.{} && !(m3 == Matrix3_Identity));
    assert(determinant3(m3) == 25, "det3 %", determinant3(m3));
    t3 := transpose(m3);
    assert(t3._12 == m3._21 && t3._31 == m3._13 && transpose(t3) == m3);
    in_place := m3;
    transpose(*in_place);
    assert(in_place == t3);
    inv3 := inverse(m3);
    assert(close(m3 * inv3, Matrix3_Identity) && close(inv3 * m3, Matrix3_Identity), "Matrix3 inverse");
    singular3 := Matrix3.{ 1, 2, 3, 2, 4, 6, 0, 0, 1 };
    assert(inverse(singular3) == Matrix3_Identity, "a singular Matrix3 inverts to the identity");
    // adjoint_transpose rows are the cofactors: m * transpose(cofactors) = det * I.
    assert(close(m3 * transpose(adjoint_transpose(m3)), Matrix3_Identity * 25));
    assert(lerp(Matrix3_Identity, m3, 0) == Matrix3_Identity && close(lerp(Matrix3_Identity, m3, 0.5), (Matrix3_Identity + m3) * 0.5));
    assert(make_matrix3(m3) == m3);
    widened := make_matrix4(m3);
    assert(widened._11 == 2 && widened._13 == 1 && widened._44 == 1 && widened._14 == 0 && widened._41 == 0);
    id3: Matrix3;
    identity(*id3);
    assert(id3 == Matrix3_Identity);
    assert(norm_inf(m3) == 5 && norm_one(m3) == 5 && mat_norm(Matrix3.{ 1, -7, 0, 2, 0, 0, 3, 0, 0 }, false) == 8 && mat_norm(Matrix3.{ 1, -7, 0, 2, 0, 0, 3, 0, 0 }, true) == 7);

    // Matrix4x3 (affine).
    a := Matrix4x3.{ 1, 0, 0, 5, 0, 2, 0, 6, 0, 0, 3, 7 };
    assert(transform_point(a, Vector3.{ 1, 1, 1 }) == Vector3.{ 6, 8, 10 } && transform_vector(a, Vector3.{ 1, 1, 1 }) == Vector3.{ 1, 2, 3 });
    assert(multiply(a, Vector4.{ 1, 1, 1, 0 }) == Vector3.{ 1, 2, 3 } && multiply(a, Vector4.{ 0, 0, 0, 1 }) == Vector3.{ 5, 6, 7 });
    aa := a * a;
    assert(transform_point(aa, Vector3.{ 1, 1, 1 }) == transform_point(a, transform_point(a, Vector3.{ 1, 1, 1 })), "4x3 product composes: %", aa);
    m4a := make_matrix4(a);
    assert(m4a._14 == 5 && m4a._34 == 7 && m4a._44 == 1 && m4a._41 == 0);
    assert(close(make_matrix4(a * a), m4a * m4a) && close(a * m4a, m4a * m4a) && close(m4a * a, m4a * m4a));
    assert(close(m4a * Matrix3_Identity, m4a) && close(m4a * make_matrix3(a), m4a * make_matrix4(make_matrix3(a))));
    assert(a + a == a * 2 && 2 * a == a * 2 && a - a == Matrix4x3.{} && !(a == Matrix4x3_Identity));
    id43: Matrix4x3;
    identity(*id43);
    assert(id43 == Matrix4x3_Identity && make_matrix3(a) == Matrix3.{ 1, 0, 0, 0, 2, 0, 0, 0, 3 });
}

matrix4_algebra :: () {
    m := Matrix4.{ 1, 2, 3, 4, 5, 6, 7, 8, 2, 6, 4, 8, 3, 1, 1, 2 };
    assert(determinant(m) == 72, "det %", determinant(m));
    assert(row(m, 1) == Vector4.{ 5, 6, 7, 8 } && column(m, 2) == Vector4.{ 3, 7, 4, 1 });
    assert(m * Vector4.{ 1, 0, 0, 0 } == column(m, 0) && multiply(m, Vector4.{ 0, 0, 0, 1 }) == column(m, 3));
    minor := minor3(m, 0, 0);
    assert(minor == Matrix3.{ 6, 7, 8, 6, 4, 8, 1, 1, 2 });
    inv, ok := inverse(m);
    assert(ok && close(m * inv, Matrix4_Identity) && close(inv * m, Matrix4_Identity), "Matrix4 inverse");
    assert(close(adjugate(m), inv * 72, 0.001) && close(adjugate_lengyel(m), inv * 72, 0.001), "adjugate = det * inverse");
    assert(close(inverse_gaussian_elimination(m), inv));
    lengyel, lengyel_ok := inverse_lengyel(m);
    assert(lengyel_ok && close(lengyel, inv));
    // A zero leading entry needs pivoting.
    swapped := Matrix4.{ 0, 1, 0, 0, 1, 0, 0, 0, 0, 0, 0, 1, 0, 0, 1, 0 };
    swapped_inverse, swapped_ok := inverse(swapped);
    assert(swapped_ok && swapped_inverse == transpose(swapped), "a permutation inverts to its transpose");
    singular := Matrix4.{ 1, 2, 3, 4, 2, 4, 6, 8, 0, 1, 0, 1, 1, 1, 1, 1 };
    _, singular_ok := inverse(singular);
    assert(!singular_ok && determinant(singular) == 0);
    _, lengyel_ok = inverse_lengyel(singular);
    assert(!lengyel_ok);
    near_singular := Matrix4_Identity * 0.05;
    _, lengyel_ok = inverse_lengyel(near_singular);
    assert(!lengyel_ok, "inverse_lengyel refuses a determinant under its epsilon");
    _, ok = inverse(near_singular);
    assert(ok, "inverse still inverts a small but regular matrix");

    t := transpose(m);
    assert(t._12 == 5 && t._41 == 4 && transpose(t) == m && close(determinant(t), 72));
    tm := m;
    transpose(*tm);
    assert(tm == t);
    assert(m + m == m * 2 && 2 * m == m * 2 && m - m == Matrix4.{} && !(m == t));
    assert(close(transpose(m * t), transpose(t) * transpose(m)), "(AB)^T = B^T A^T");
    id4: Matrix4;
    identity(*id4);
    assert(id4 == Matrix4_Identity && identity_of(Matrix4) == Matrix4_Identity && make_matrix4(m) == m);
    assert(m.coef[3][0] == 3 && m.v[2] == Vector4.{ 2, 6, 4, 8 } && m.floats[15] == 2);
}

transforms :: () {
    // Scale, translate and rotate compose like their matrices.
    s := make_scale_matrix4(Vector3.{ 2, 3, 4 });
    assert(s * Vector4.{ 1, 1, 1, 1 } == Vector4.{ 2, 3, 4, 1 });
    tr := make_translation_matrix4(Vector3.{ 10, 20, 30 });
    assert(tr * Vector4.{ 1, 1, 1, 1 } == Vector4.{ 11, 21, 31, 1 } && tr * Vector4.{ 1, 1, 1, 0 } == Vector4.{ 1, 1, 1, 0 });
    moved := translate(Matrix4_Identity, Vector3.{ 1, 2, 3 });
    assert(moved == make_translation_matrix4(Vector3.{ 1, 2, 3 }));
    translate(*moved, Vector3.{ 1, 1, 1 });
    assert(moved._14 == 2 && moved._24 == 3 && moved._34 == 4);
    scaled := scale(Matrix4_Identity, 2.0);
    assert(scaled._11 == 2 && scaled._22 == 2 && scaled._33 == 2 && scaled._44 == 1);
    scaled = scale(tr, Vector3.{ 1, 2, 3 });
    assert(scaled._22 == 2 && scaled._33 == 3 && scaled._14 == 10, "scale leaves the translation alone");
    scale(*scaled, 0.5);
    assert(scaled._22 == 1 && scaled._11 == 0.5);
    scaled3 := scale(Matrix3_Identity, Vector3.{ 4, 5, 6 });
    assert(scaled3 == Matrix3.{ 4, 0, 0, 0, 5, 0, 0, 0, 6 });

    q: Quaternion;
    set_from_axis_and_angle(*q, Vector3.{ 0, 1, 0 }, PI / 2);
    r3 := rotation_matrix(Matrix3, q);
    assert(close(r3 * Vector3.{ 1, 0, 0 }, rotate(Vector3.{ 1, 0, 0 }, q)) && close(r3 * Vector3.{ 1, 0, 0 }, Vector3.{ 0, 0, -1 }), "rotation matrix about y");
    assert(close(determinant3(r3), 1) && close(r3 * transpose(r3), Matrix3_Identity), "rotation matrices are orthogonal");
    r4 := rotation_matrix(Matrix4, q);
    assert(close(r4 * Vector4.{ 0, 0, 1, 1 }, Vector4.{ 1, 0, 0, 1 }) && r4._44 == 1);
    assert(close(rotate(Matrix3_Identity, q), r3) && close(rotate(Matrix4_Identity, q), r4) && close(rotate(r3, conjugate(q)), Matrix3_Identity));
    set_target: Matrix4x3;
    identity(*set_target);
    set_rotation(*set_target, q);
    assert(close(transform_vector(set_target, Vector3.{ 1, 0, 0 }), Vector3.{ 0, 0, -1 }));
    unnormalized := Quaternion.{ q.x * 3, q.y * 3, q.z * 3, q.w * 3 };
    assert(close(rotation_matrix(Matrix3, unnormalized), r3), "set_rotation normalizes its quaternion");

    // get_rotation inverts rotation_matrix in every branch (w largest, then x, y or z largest).
    angles := float.[0.3, 3.0, 3.0, 3.0, PI];
    axes := Vector3.[.{ 1, 2, 3 }, .{ 1, 0.1, 0.1 }, .{ 0.1, 1, 0.1 }, .{ 0.1, 0.1, 1 }, .{ 0, 0, 1 }];
    for angle, index: angles {
        original: Quaternion;
        set_from_axis_and_angle(*original, axes[index], angle);
        recovered := get_rotation(rotation_matrix(Matrix3, original));
        assert(same_rotation(recovered, original), "get_rotation case %: % vs %", index, recovered, original);
        recovered = get_rotation(rotation_matrix(Matrix4, original));
        assert(same_rotation(recovered, original), "get_rotation from a Matrix4, case %", index);
    }

    // Basis matrices.
    xp := Vector3.{ 0, 1, 0 };
    yp := Vector3.{ -1, 0, 0 };
    zp := Vector3.{ 0, 0, 1 };
    rows := make_matrix_from_rows(xp, yp, zp);
    columns := make_matrix_from_columns(xp, yp, zp);
    assert(rows.v[0].xyz == xp && rows.v[1].xyz == yp && rows._44 == 1 && columns == transpose(rows));
    rows3 := make_matrix_from_rows(xp, yp, zp, Matrix3);
    columns3 := make_matrix_from_columns(xp, yp, zp, Matrix3);
    assert(rows3.v[0] == xp && columns3 == transpose(rows3) && columns3 * Vector3.{ 1, 0, 0 } == xp);

    // Rigid transforms invert by transposing the rotation.
    rigid := make_translation_matrix4(Vector3.{ 1, -2, 3 }) * r4;
    assert(close(isometry_inverse(rigid) * rigid, Matrix4_Identity) && close(rigid * isometry_inverse(rigid), Matrix4_Identity));

    // Look-at: the eye goes to the origin and the target onto the forward axis.
    eye := Vector3.{ 1, 2, 3 };
    target := Vector3.{ 4, 6, 3 };
    view := make_look_at_matrix(eye, target, Vector3.{ 0, 0, 1 });
    assert(close(view * xyzw(eye, 1), Vector4.{ 0, 0, 0, 1 }) && close(view * xyzw(target, 1), Vector4.{ 5, 0, 0, 1 }), "x forward: %", view * xyzw(target, 1));
    view = make_look_at_matrix(eye, target, Vector3.{ 0, 0, 1 }, x_is_forward = false);
    assert(close(view * xyzw(eye, 1), Vector4.{ 0, 0, 0, 1 }) && close(view * xyzw(target, 1), Vector4.{ 0, 0, -5, 1 }), "-z forward: %", view * xyzw(target, 1));
    above := view * xyzw(eye + Vector3.{ 0, 0, 1 }, 1);
    assert(close(above, Vector4.{ 0, 1, 0, 1 }), "up stays up: %", above);
    // The -z forward view is right-handed: the camera's right is +x.
    right := view * xyzw(eye + Vector3.{ 0.8, -0.6, 0 }, 1);
    assert(close(right, Vector4.{ 1, 0, 0, 1 }), "right is +x: %", right);

    // Projections map the view volume's corners onto the clip cube.
    ortho := orthographic_projection_matrix(-2, 2, -1, 1, 1, 10);
    assert(close(ortho * Vector4.{ -2, -1, -1, 1 }, Vector4.{ -1, -1, -1, 1 }) && close(ortho * Vector4.{ 2, 1, -10, 1 }, Vector4.{ 1, 1, 1, 1 }));
    ortho = orthographic_projection_matrix(0, 4, 0, 2, 1, 10, depth_range_01 = true);
    assert(close(ortho * Vector4.{ 0, 0, -1, 1 }, Vector4.{ -1, -1, 0, 1 }) && close(ortho * Vector4.{ 4, 2, -10, 1 }, Vector4.{ 1, 1, 1, 1 }));
    frustum := make_frustum_matrix(-1, 1, -0.5, 0.5, 1, 100);
    clip := frustum * Vector4.{ -1, -0.5, -1, 1 };
    assert(close(clip.xyz / clip.w, Vector3.{ -1, -1, -1 }), "near corner: %", clip);
    clip = frustum * Vector4.{ 100, 50, -100, 1 };
    assert(close(clip.xyz / clip.w, Vector3.{ 1, 1, 1 }), "far corner: %", clip);
    frustum = make_frustum_matrix(-1, 1, -1, 1, 1, 100, depth_range_01 = true);
    clip = frustum * Vector4.{ 0, 0, -1, 1 };
    assert(close(clip.z / clip.w, 0) && clip.w == 1);
    clip = frustum * Vector4.{ 0, 0, -100, 1 };
    assert(close(clip.z / clip.w, 1));
    projection := make_projection_matrix(PI / 2, 2, 1, 100);
    clip = projection * Vector4.{ 2, 1, -1, 1 };
    assert(close(clip.xyz / clip.w, Vector3.{ 1, 1, -1 }), "projection corner: %", clip);
    shifted := make_projection_matrix(PI / 2, 2, 1, 100, x_offset = 0.25, y_offset = -0.5);
    assert(shifted._13 == 0.25 && shifted._23 == -0.5);
}

decompositions :: () {
    // Polar decomposition of rotation * stretch recovers both factors.
    q: Quaternion;
    set_from_axis_and_angle(*q, Vector3.{ 1, 1, 0 }, 0.7);
    rotation := rotation_matrix(Matrix3, q);
    stretch := Matrix3.{ 2, 0, 0, 0, 3, 0, 0, 0, 4 };
    orthogonal, symmetric, det := polar_decomp(rotation * stretch);
    assert(close(orthogonal, rotation, 0.001) && close(symmetric, stretch, 0.001) && close(det, 24, 0.001), "polar: % % %", orthogonal, symmetric, det);
    assert(close(orthogonal * symmetric, rotation * stretch, 0.001));
    flat := Matrix3.{ 1, 0, 0, 0, 1, 0, 0, 0, 0 };
    orthogonal, symmetric, det = polar_decomp(flat);
    assert(det == 0 && close(orthogonal * transpose(orthogonal), Matrix3_Identity), "a singular input still yields an orthonormal factor");

    // A Householder reflector maps v onto the x axis.
    v := Vector3.{ 3, 4, 0 };
    u := make_reflector(v);
    assert(close(length(u), 1));
    m := Matrix3.{ 3, 1, 0, 4, 1, 0, 0, 1, 1 }; // v is the first column
    reflect_cols(*m, u);
    assert(close(m._11, -5) && close(m._21, 0) && close(m._31, 0), "reflect_cols: %", m);
    m = Matrix3.{ 3, 4, 0, 1, 1, 0, 0, 0, 1 }; // v is the first row
    reflect_rows(*m, u);
    assert(close(m._11, -5) && close(m._12, 0) && close(m._13, 0), "reflect_rows: %", m);
    u = make_reflector(Vector3.{ -3, 0, 4 });
    reflected := Vector3.{ -3, 0, 4 } - u * (2 * dot(Vector3.{ -3, 0, 4 }, u));
    assert(close(reflected, Vector3.{ 5, 0, 0 }), "a negative leading entry reflects to +|v|: %", reflected);
}

planes :: () {
    p := make_plane(0, 0, 1, -5);
    assert(get_normal(p) == Vector3.{ 0, 0, 1 } && p.d == -5 && p.vector4 == Vector4.{ 0, 0, 1, -5 });
    assert(plane_dot(p, Vector3.{ 7, 8, 5 }) == 0 && plane_dot(p, Vector3.{ 0, 0, 7 }) == 2);
    assert(make_plane(Vector3.{ 0, 1, 0 }, 2) == make_plane(0, 1, 0, 2) && make_plane(Vector4.{ 1, 0, 0, 3 }) == make_plane(1, 0, 0, 3));
    through := make_plane(Vector3.{ 0, 2, 0 }, Vector3.{ 1, 1, 1 }); // normal, point
    assert(close(plane_dot(through, Vector3.{ 1, 1, 1 }), 0) && close(plane_dot(through, Vector3.{ 1, 2, 1 }), 2));
    from_point := plane_from_point_and_normal(Vector3.{ 0, 0, 3 }, Vector3.{ 0, 0, 1 });
    assert(from_point == make_plane(0, 0, 1, -3));
    unit := normalize(make_plane(0, 3, 4, 10));
    assert(close(unit.a, 0) && close(unit.b, 0.6) && close(unit.c, 0.8) && close(unit.d, 2));
    degenerate := make_plane(0, 0, 0, 1);
    assert(normalize(degenerate) == degenerate);
    tri := normalized_plane_from_points(Vector3.{ 0, 0, 1 }, Vector3.{ 1, 0, 1 }, Vector3.{ 0, 1, 1 });
    assert(close(tri.normal, Vector3.{ 0, 0, 1 }) && close(tri.d, -1), "plane through three points: %", tri);
    for Vector3.[.{ 0, 0, 1 }, .{ 1, 0, 1 }, .{ 0, 1, 1 }] assert(abs(plane_dot(tri, it)) < EPS);
}
