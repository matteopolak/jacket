row :: (m: Matrix4, i: int) -> Vector4 {
    return m.v[i];
}

column :: (m: Matrix4, i: int) -> Vector4 {
    return .{ m.coef[0][i], m.coef[1][i], m.coef[2][i], m.coef[3][i] };
}

multiply :: (m: Matrix4, v: Vector4) -> Vector4 {
    return .{ dot(m.v[0], v), dot(m.v[1], v), dot(m.v[2], v), dot(m.v[3], v) };
}

multiply :: (m: Matrix4x3, v: Vector4) -> Vector3 {
    return .{ dot(m.v[0], v), dot(m.v[1], v), dot(m.v[2], v) };
}

multiply :: (m: Matrix2, v: Vector2) -> Vector2 {
    return .{ dot(m.v[0], v), dot(m.v[1], v) };
}

operator * :: (a: Matrix2, b: Vector2) -> Vector2 {
    return multiply(a, b);
}

multiply :: (m: $T/interface AnyMatrix3, v: Vector3) -> Vector3 {
    return .{ m._11 * v.x + m._12 * v.y + m._13 * v.z, m._21 * v.x + m._22 * v.y + m._23 * v.z, m._31 * v.x + m._32 * v.y + m._33 * v.z };
}

transform_vector :: (m: $T/interface AnyMatrix3, v: Vector3) -> Vector3 {
    return multiply(m, v);
}

transform_point :: (m: $T/interface AnyMatrix4x3, v: Vector3) -> Vector3 {
    r := multiply(m, v);
    return r + xyz(m._14, m._24, m._34);
}

multiply :: (m: Matrix2, n: Matrix2) -> Matrix2 {
    r: Matrix2;
    for i: 0..1 for j : 0..1 for k : 0..1 r.coef[i][j] += m.coef[i][k] * n.coef[k][j];
    return r;
}

multiply :: (m: Matrix3, n: Matrix3) -> Matrix3 {
    r: Matrix3;
    for i: 0..2 for j : 0..2 for k : 0..2 r.coef[i][j] += m.coef[i][k] * n.coef[k][j];
    return r;
}

multiply :: (m: Matrix4, n: Matrix4) -> Matrix4 #no_alias {
    r: Matrix4;
    for i: 0..3 for j : 0..3 for k : 0..3 r.coef[i][j] += m.coef[i][k] * n.coef[k][j];
    return r;
}

multiply :: (m: Matrix4, n: Matrix3) -> Matrix4 {
    return multiply(m, make_matrix4(n));
}

multiply :: (m: Matrix4, n: Matrix4x3) -> Matrix4 {
    r := make_matrix4(n);
    r._14 = n._14;
    r._24 = n._24;
    r._34 = n._34;

    return multiply(m, r);
}

multiply :: (m: Matrix4x3, n: Matrix4) -> Matrix4 {
    r := make_matrix4(m);
    r._14 = m._14;
    r._24 = m._24;
    r._34 = m._34;

    return multiply(r, n);
}

multiply :: (m: Matrix4x3, n: Matrix4x3) -> Matrix4x3 {
    r := multiply(m, make_affine4(n));
    return r.mat4x3;
}

make_matrix3 :: (m: $T/interface AnyMatrix3) -> Matrix3 {
    return .{ m._11, m._12, m._13, m._21, m._22, m._23, m._31, m._32, m._33 };
}

make_matrix4 :: (m: $T/interface AnyMatrix3) -> Matrix4 {
    return .{ m._11, m._12, m._13, 0, m._21, m._22, m._23, 0, m._31, m._32, m._33, 0, 0, 0, 0, 1 };
}

transpose :: (m: Matrix3) -> Matrix3 {
    r: Matrix3;
    for i: 0..2 for j : 0..2 r.coef[i][j] = m.coef[j][i];
    return r;
}

transpose :: (m: Matrix4) -> Matrix4 {
    r: Matrix4;
    for i: 0..3 for j : 0..3 r.coef[i][j] = m.coef[j][i];
    return r;
}

transpose :: (m: *Matrix3) {
    m.* = transpose(m.*);
}

transpose :: (m: *Matrix4) {
    m.* = transpose(m.*);
}

identity_of :: ($T: Type) -> T {
    r: T;
    identity(*r);
    return r;
}

identity :: (m: *$T/interface AnyMatrix) {
    m.* = .{};
    m._11 = 1;
    m._22 = 1;
    #if T != Matrix2 m._33 = 1;
    #if T == Matrix4 m._44 = 1;
}

make_scale_matrix4 :: (v: Vector3) -> Matrix4 {
    r := Matrix4_Identity;
    r._11 = v.x;
    r._22 = v.y;
    r._33 = v.z;

    return r;
}

scale :: (m: $T/interface AnyMatrix3, s: float) -> T {
    return scale(m, xyz(s));
}

scale :: (m: *$T/interface AnyMatrix3, s: float) {
    m.* = scale(m.*, s);
}

scale :: (m: $T/interface AnyMatrix3, v: Vector3) -> T {
    r := m;
    r._11 *= v.x;
    r._21 *= v.x;
    r._31 *= v.x;
    r._12 *= v.y;
    r._22 *= v.y;
    r._32 *= v.y;
    r._13 *= v.z;
    r._23 *= v.z;
    r._33 *= v.z;

    #if T == Matrix4 {
        r._41 *= v.x;
        r._42 *= v.y;
        r._43 *= v.z;
    }

    return r;
}

scale :: (m: *$T/interface AnyMatrix3, v: Vector3) {
    m.* = scale(m.*, v);
}

make_translation_matrix4 :: (v: Vector3) -> Matrix4 {
    r := Matrix4_Identity;
    r._14 = v.x;
    r._24 = v.y;
    r._34 = v.z;

    return r;
}

translate :: (using m: Matrix4, t: Vector3) -> Matrix4 {
    r := m;
    r._14 += t.x;
    r._24 += t.y;
    r._34 += t.z;

    return r;
}

translate :: (using m: *Matrix4, t: Vector3) {
    m.* = translate(m.*, t);
}

rotate :: (m: Matrix3, q: Quaternion) -> Matrix3 {
    return multiply(m, rotation_matrix(Matrix3, q));
}

rotate :: (m: Matrix4, q: Quaternion) -> Matrix4 {
    return multiply(m, rotation_matrix(Matrix4, q));
}

make_matrix_from_rows :: (xprime: Vector3, yprime: Vector3, zprime: Vector3, $T := Matrix4) -> T {
    r := identity_of(T);
    r._11 = xprime.x;
    r._12 = xprime.y;
    r._13 = xprime.z;
    r._21 = yprime.x;
    r._22 = yprime.y;
    r._23 = yprime.z;
    r._31 = zprime.x;
    r._32 = zprime.y;
    r._33 = zprime.z;

    return r;
}

make_matrix_from_columns :: (xprime: Vector3, yprime: Vector3, zprime: Vector3, $T := Matrix4) -> T {
    return make_matrix_from_rows(xyz(xprime.x, yprime.x, zprime.x), xyz(xprime.y, yprime.y, zprime.y), xyz(xprime.z, yprime.z, zprime.z), T);
}

make_look_at_matrix :: (viewpoint: Vector3, look_at: Vector3, reference_up_vector: Vector3, $x_is_forward := true) -> Matrix4 {
    f := unit_vector(look_at - viewpoint);
    side := unit_vector(cross(f, reference_up_vector));
    up := cross(side, f);
    r: Matrix4;

    #if x_is_forward r = make_matrix_from_rows(f, -side, up);
    else r = make_matrix_from_rows(side, up, -f);
    r._14 = -dot(r.v[0].xyz, viewpoint);
    r._24 = -dot(r.v[1].xyz, viewpoint);
    r._34 = -dot(r.v[2].xyz, viewpoint);

    return r;
}

orthographic_projection_matrix :: (left: float, right: float, bottom: float, top: float, near: float, far: float, depth_range_01 := false) -> Matrix4 {
    r := Matrix4_Identity;
    r._11 = 2 / (right - left);
    r._22 = 2 / (top - bottom);
    r._14 = -(right + left) / (right - left);
    r._24 = -(top + bottom) / (top - bottom);

    if depth_range_01 {
        r._33 = -1 / (far - near);
        r._34 = -near / (far - near);
    } else {
        r._33 = -2 / (far - near);
        r._34 = -(far + near) / (far - near);
    }

    return r;
}

make_frustum_matrix :: (l: float, r: float, b: float, t: float, n: float, f: float, depth_range_01 := false) -> Matrix4 {
    m: Matrix4;
    m._11 = 2 * n / (r - l);
    m._22 = 2 * n / (t - b);
    m._13 = (r + l) / (r - l);
    m._23 = (t + b) / (t - b);
    m._43 = -1;

    if depth_range_01 {
        m._33 = -f / (f - n);
        m._34 = -f * n / (f - n);
    } else {
        m._33 = -(f + n) / (f - n);
        m._34 = -2 * f * n / (f - n);
    }

    return m;
}

make_projection_matrix :: (fov_vertical: float, aspect_ratio_horizontal_over_vertical: float, z_near: float, z_far: float, x_offset := 0.0, y_offset := 0.0, depth_range_01 := false) -> Matrix4 {
    t := z_near * tan(fov_vertical / 2);
    r := t * aspect_ratio_horizontal_over_vertical;
    m := make_frustum_matrix(-r, r, -t, t, z_near, z_far, depth_range_01);
    m._13 += x_offset;
    m._23 += y_offset;

    return m;
}

lerp :: (a: Matrix3, b: Matrix3, t: float) -> Matrix3 {
    r: Matrix3;
    for i: 0..8 r.floats[i] = lerp(a.floats[i], b.floats[i], t);
    return r;
}

adjoint_transpose :: (m: Matrix3) -> Matrix3 {
    return make_matrix_from_rows(cross(m.v[1], m.v[2]), cross(m.v[2], m.v[0]), cross(m.v[0], m.v[1]), Matrix3);
}

inverse :: (a: Matrix3, epsilon := 0.001) -> (success: bool, result: Matrix3) {
    co := adjoint_transpose(a);
    d := dot(a.v[0], co.v[0]);

    if d > -epsilon && d < epsilon return false, a;

    r := transpose(co);
    for *x: r.floats x.* /= d;

    return true, r;
}

inverse :: (m: Matrix4) -> (success: bool, result: Matrix4) {
    return inverse_gaussian_elimination(m);
}

inverse_gaussian_elimination :: (m: Matrix4) -> (success: bool, result: Matrix4) {
    a := m;
    b := Matrix4_Identity;

    for k: 0..3 {
        pivot := k;
        for i: k..3 if abs(a.coef[i][k]) > abs(a.coef[pivot][k]) pivot = i;
        if a.coef[pivot][k] == 0 return false, Matrix4_Identity;

        if pivot != k {
            swap(*a.v[k], *a.v[pivot]);
            swap(*b.v[k], *b.v[pivot]);
        }

        s := a.coef[k][k];

        for j: 0..3 {
            a.coef[k][j] /= s;
            b.coef[k][j] /= s;
        }

        for i: 0..3 if i != k {
            factor := a.coef[i][k];

            for j: 0..3 {
                a.coef[i][j] -= factor * a.coef[k][j];
                b.coef[i][j] -= factor * b.coef[k][j];
            }
        }
    }

    return true, b;
}

adjugate_lengyel :: (m: Matrix4) -> Matrix4 {
    r: Matrix4;

    for i: 0..3 for j : 0..3 {
        s := minor4(m, j, i);
        if ((i + j) & 1) != 0 s = -s;
        r.coef[i][j] = s;
    }

    return r;
}

adjugate :: adjugate_lengyel;

determinant :: (m: Matrix4) -> float {
    d: float = 0;

    for j: 0..3 {
        term := m.coef[0][j] * minor4(m, 0, j);
        if (j & 1) == 0 d += term;
        else d -= term;
    }

    return d;
}

inverse_lengyel :: (m: Matrix4, epsilon := 0.0001) -> (success: bool, result: Matrix4) {
    d := determinant(m);
    if abs(d) < epsilon return false, Matrix4_Identity;

    r := adjugate(m);
    for *x: r.floats x.* /= d;

    return true, r;
}

isometry_inverse :: (m: Matrix4) -> Matrix4 {
    r := make_matrix4(transpose(make_matrix3(m)));
    t := -multiply(r, xyz(m._14, m._24, m._34));

    r._14 = t.x;
    r._24 = t.y;
    r._34 = t.z;

    return r;
}

mat_norm :: (M: Matrix3, tpose: bool) -> float {
    result: float = 0;

    for i: 0..2 {
        s: float = 0;

        for j: 0..2 {
            if tpose s += abs(M.coef[j][i]);
            else s += abs(M.coef[i][j]);
        }

        result = max(result, s);
    }

    return result;
}

norm_inf :: #bake_arguments mat_norm(tpose = false);
norm_one :: #bake_arguments mat_norm(tpose = true);

make_reflector :: (v: Vector3) -> (u: Vector3) {
    u := v;
    n := length(v);

    if n == 0 return .{};
    if v.z >= 0 u.z += n;
    else u.z -= n;

    return normalize(u) * cast(float) SQRT2;
}

reflect_cols :: (M: *Matrix3, u: Vector3) {
    for i: 0..2 M.v[i] -= u * dot(M.v[i], u);
}

reflect_rows :: (M: *Matrix3, u: Vector3) {
    r := transpose(M.*);
    reflect_cols(*r, u);
    M.* = transpose(r);
}

polar_decomp :: (M: Matrix3) -> (Q: Matrix3, S: Matrix3, det: float) {
    // Jacobi eigensystem of M^T M: singular values, then an orthogonal completion.
    c := multiply(transpose(M), M);
    v := Matrix3_Identity;

    for sweep: 0..31 {
        for p: 0..1 for q : p + 1..2 {
            a := c.coef[p][q];
            if abs(a) <= 0.00000001 * sqrt(abs(c.coef[p][p] * c.coef[q][q])) continue;

            theta := 0.5 * atan2(2 * a, c.coef[q][q] - c.coef[p][p]);
            cs := cos(theta);
            sn := sin(theta);
            j := Matrix3_Identity;
            j.coef[p][p] = cs;
            j.coef[q][q] = cs;
            j.coef[p][q] = sn;
            j.coef[q][p] = -sn;
            c = multiply(transpose(j), multiply(c, j));
            v = multiply(v, j);
        }
    }

    d: Matrix3;
    invd: Matrix3;

    for i: 0..2 {
        d.coef[i][i] = sqrt(max(c.coef[i][i], 0.0));
        if d.coef[i][i] > 0.0000001 invd.coef[i][i] = 1 / d.coef[i][i];
    }

    u := multiply(multiply(M, v), invd);

    // Fill null-space directions without disturbing nonzero singular vectors.
    for i: 0..2 if d.coef[i][i] <= 0.0000001 {
        best: Vector3;
        bestn: float = -1;

        for axis: 0..2 {
            candidate: Vector3;
            candidate.component[axis] = 1;

            for k: 0..2 if k != i {
                col := xyz(u.coef[0][k], u.coef[1][k], u.coef[2][k]);
                candidate -= col * dot(candidate, col);
            }

            nn := length_squared(candidate);
            if nn > bestn {
                bestn = nn;
                best = candidate;
            }
        }

        best = unit_vector(best);
        for row: 0..2 u.coef[row][i] = best.component[row];
    }

    Q = multiply(u, transpose(v));
    S = multiply(multiply(v, d), transpose(v));
    det = dot(M.v[0], cross(M.v[1], M.v[2]));

    return Q, S, det;
}

operator * :: (m: Matrix4, v: Vector4) -> Vector4 {
    return multiply(m, v);
}

operator * :: (m: Matrix4x3, v: Vector4) -> Vector3 {
    return multiply(m, v);
}

operator * :: (m: Matrix3, v: Vector3) -> Vector3 {
    return multiply(m, v);
}

operator * :: (a: Matrix4, b: Matrix4) -> Matrix4 #no_alias {
    return multiply(a, b);
}

operator * :: (a: Matrix4x3, b: Matrix4x3) -> Matrix4x3 {
    return multiply(a, b);
}

operator * :: (a: Matrix4, b: Matrix4x3) -> Matrix4 {
    return multiply(a, b);
}

operator * :: (a: Matrix4x3, b: Matrix4) -> Matrix4 {
    return multiply(a, b);
}

operator * :: (a: Matrix4, b: Matrix3) -> Matrix4 {
    return multiply(a, b);
}

operator * :: (a: Matrix3, b: Matrix3) -> Matrix3 {
    return multiply(a, b);
}

operator * :: (a: Matrix2, b: Matrix2) -> Matrix2 {
    return multiply(a, b);
}

operator * :: (a: $T/interface AnyMatrix, t: float) -> T #symmetric {
    result := a;
    for *value: result.floats value.* *= t;
    return result;
}

operator + :: (a: $T/interface AnyMatrix, b: T) -> T {
    result := a;
    for *value, index: result.floats value.* += b.floats[index];
    return result;
}

operator - :: (a: $T/interface AnyMatrix, b: T) -> T {
    result := a;
    for *value, index: result.floats value.* -= b.floats[index];
    return result;
}

#scope_file
#import "Basic";

make_affine4 :: (m: Matrix4x3) -> Matrix4 {
    r := make_matrix4(m);
    r._14 = m._14;
    r._24 = m._24;
    r._34 = m._34;

    return r;
}

minor4 :: (m: Matrix4, row: int, col: int) -> float {
    a: Matrix3;
    ii := 0;

    for i: 0..3 if i != row {
        jj := 0;

        for j: 0..3 if j != col {
            a.coef[ii][jj] = m.coef[i][j];
            jj += 1;
        }

        ii += 1;
    }

    return dot(a.v[0], cross(a.v[1], a.v[2]));
}
