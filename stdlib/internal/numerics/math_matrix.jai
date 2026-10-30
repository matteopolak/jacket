// The `i`th row of the matrix, counting from 0.
//
// - m: the matrix.
// - i: the row index, 0 to 3.
row :: (m: Matrix4, i: int) -> Vector4 {
    return m.v[i];
}

// The `i`th column of the matrix, counting from 0.
//
// - m: the matrix.
// - i: the column index, 0 to 3.
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

operator == :: (a: Matrix2, b: Matrix2) -> bool {
    for 0..3 if a.floats[it] != b.floats[it] return false;
    return true;
}

operator == :: (a: Matrix3, b: Matrix3) -> bool {
    for 0..8 if a.floats[it] != b.floats[it] return false;
    return true;
}

operator == :: (a: Matrix4, b: Matrix4) -> bool {
    for 0..15 if a.floats[it] != b.floats[it] return false;
    return true;
}

operator * :: (a: Matrix2, b: Vector2) -> Vector2 {
    return multiply(a, b);
}

multiply :: (m: $T/interface AnyMatrix3, v: Vector3) -> Vector3 {
    return .{ m._11 * v.x + m._12 * v.y + m._13 * v.z, m._21 * v.x + m._22 * v.y + m._23 * v.z, m._31 * v.x + m._32 * v.y + m._33 * v.z };
}

// Applies the matrix to `v`, ignoring any translation.
//
// - m: the matrix.
// - v: the vector to transform.
transform_vector :: (m: $T/interface AnyMatrix3, v: Vector3) -> Vector3 {
    return multiply(m, v);
}

// Applies the matrix to `v` and then adds its translation column.
//
// - m: the matrix.
// - v: the point to transform.
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

// The upper-left 3x3 part of `m`.
//
// - m: the matrix to copy from.
make_matrix3 :: (m: $T/interface AnyMatrix3) -> Matrix3 {
    return .{ m._11, m._12, m._13, m._21, m._22, m._23, m._31, m._32, m._33 };
}

// `m` as a 4x4 matrix. The extra row and column are those of the identity.
//
// - m: the matrix to copy from.
make_matrix4 :: (m: $T/interface AnyMatrix3) -> Matrix4 {
    return .{ m._11, m._12, m._13, 0, m._21, m._22, m._23, 0, m._31, m._32, m._33, 0, 0, 0, 0, 1 };
}

// The matrix with rows and columns swapped.
//
// - m: the matrix.
transpose :: (m: Matrix3) -> Matrix3 {
    r: Matrix3;
    for i: 0..2 for j : 0..2 r.coef[i][j] = m.coef[j][i];
    return r;
}

// The matrix with rows and columns swapped.
//
// - m: the matrix.
transpose :: (m: Matrix4) -> Matrix4 {
    r: Matrix4;
    for i: 0..3 for j : 0..3 r.coef[i][j] = m.coef[j][i];
    return r;
}

// Swaps rows and columns in place.
//
// - m: the matrix to change.
transpose :: (m: *Matrix3) {
    m.* = transpose(m.*);
}

// Swaps rows and columns in place.
//
// - m: the matrix to change.
transpose :: (m: *Matrix4) {
    m.* = transpose(m.*);
}

// An identity matrix of type `T`.
//
// - T: the matrix type.
identity_of :: ($T: Type) -> T {
    r: T;
    identity(*r);
    return r;
}

// Sets the matrix to the identity, in place.
//
// - m: the matrix to change.
identity :: (m: *$T/interface AnyMatrix) {
    m.* = .{};
    m._11 = 1;
    m._22 = 1;
    #if T != Matrix2 m._33 = 1;
    #if T == Matrix4 m._44 = 1;
}

// A matrix that scales each axis by the matching part of `v`.
//
// - v: the scale on x, y and z.
make_scale_matrix4 :: (v: Vector3) -> Matrix4 {
    r := Matrix4_Identity;
    r._11 = v.x;
    r._22 = v.y;
    r._33 = v.z;

    return r;
}

// `m` scaled by the same factor on all three axes.
//
// - m: the matrix.
// - s: the scale factor.
scale :: (m: $T/interface AnyMatrix3, s: float) -> T {
    return scale(m, xyz(s));
}

// Scales the matrix by the same factor on all three axes, in place.
//
// - m: the matrix to change.
// - s: the scale factor.
scale :: (m: *$T/interface AnyMatrix3, s: float) {
    m.* = scale(m.*, s);
}

// `m` scaled per axis.
//
// - m: the matrix.
// - v: the scale on x, y and z.
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

// Scales the matrix per axis, in place.
//
// - m: the matrix to change.
// - v: the scale on x, y and z.
scale :: (m: *$T/interface AnyMatrix3, v: Vector3) {
    m.* = scale(m.*, v);
}

// A matrix that moves points by `v`.
//
// - v: the offset on x, y and z.
make_translation_matrix4 :: (v: Vector3) -> Matrix4 {
    r := Matrix4_Identity;
    r._14 = v.x;
    r._24 = v.y;
    r._34 = v.z;

    return r;
}

// `m` with `t` added to its translation.
//
// - m: the matrix.
// - t: the offset on x, y and z.
translate :: (using m: Matrix4, t: Vector3) -> Matrix4 {
    r := m;
    r._14 += t.x;
    r._24 += t.y;
    r._34 += t.z;

    return r;
}

// Adds `t` to the translation of the matrix, in place.
//
// - m: the matrix to change.
// - t: the offset on x, y and z.
translate :: (using m: *Matrix4, t: Vector3) {
    m.* = translate(m.*, t);
}

rotate :: (m: Matrix3, q: Quaternion) -> Matrix3 {
    return multiply(m, rotation_matrix(Matrix3, q));
}

rotate :: (m: Matrix4, q: Quaternion) -> Matrix4 {
    return multiply(m, rotation_matrix(Matrix4, q));
}

// A matrix whose upper-left 3x3 has the given vectors as its rows. The rest is the identity.
//
// - xprime: the first row.
// - yprime: the second row.
// - zprime: the third row.
// - T: the matrix type to make.
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

// A matrix whose upper-left 3x3 has the given vectors as its columns. The rest is the identity.
//
// - xprime: the first column.
// - yprime: the second column.
// - zprime: the third column.
// - T: the matrix type to make.
make_matrix_from_columns :: (xprime: Vector3, yprime: Vector3, zprime: Vector3, $T := Matrix4) -> T {
    return make_matrix_from_rows(xyz(xprime.x, yprime.x, zprime.x), xyz(xprime.y, yprime.y, zprime.y), xyz(xprime.z, yprime.z, zprime.z), T);
}

// A view matrix for a camera at `viewpoint` looking at `look_at`.
//
// - viewpoint: where the camera is.
// - look_at: the point it looks at.
// - reference_up_vector: roughly which way is up. Must not be parallel to the view direction.
// - x_is_forward: true makes the camera look down the x axis, false down the negative z axis.
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

// An orthographic projection for the box between the six planes.
//
// - left: the left plane.
// - right: the right plane.
// - bottom: the bottom plane.
// - top: the top plane.
// - near: distance to the near plane.
// - far: distance to the far plane.
// - depth_range_01: true maps depth to `0..1`, false to `-1..1`.
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

// A perspective projection from the rectangle on the near plane.
//
// - l: left edge of the near rectangle.
// - r: right edge of the near rectangle.
// - b: bottom edge of the near rectangle.
// - t: top edge of the near rectangle.
// - n: distance to the near plane.
// - f: distance to the far plane.
// - depth_range_01: true maps depth to `0..1`, false to `-1..1`.
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

// A perspective projection from a field of view.
//
// - fov_vertical: the vertical field of view in radians.
// - aspect_ratio_horizontal_over_vertical: width divided by height.
// - z_near: distance to the near plane.
// - z_far: distance to the far plane.
// - x_offset: shifts the view centre horizontally.
// - y_offset: shifts the view centre vertically.
// - depth_range_01: true maps depth to `0..1`, false to `-1..1`.
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
} @UseResult

// The transpose of the adjugate: its rows are the cross products of the other two rows.
//
// - m: the matrix.
adjoint_transpose :: (m: Matrix3) -> Matrix3 {
    return make_matrix_from_rows(cross(m.v[1], m.v[2]), cross(m.v[2], m.v[0]), cross(m.v[0], m.v[1]), Matrix3);
}

// The inverse, or false and the input back when there is none.
//
// - a: the matrix.
// - epsilon: a determinant closer to 0 than this counts as having no inverse.
inverse :: (a: Matrix3, epsilon := 0.001) -> (success: bool, result: Matrix3) {
    co := adjoint_transpose(a);
    d := dot(a.v[0], co.v[0]);

    if d > -epsilon && d < epsilon return false, a;

    r := transpose(co);
    for *x: r.floats x.* /= d;

    return true, r;
}

// The inverse, or false and the identity when there is none.
//
// - m: the matrix.
inverse :: (m: Matrix4) -> (success: bool, result: Matrix4) {
    return inverse_gaussian_elimination(m);
}

// The inverse by Gauss-Jordan elimination with row pivoting. False and the identity when there
// is none.
//
// - m: the matrix.
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

// The adjugate (transposed cofactor matrix).
//
// - m: the matrix.
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

// The determinant.
//
// - m: the matrix.
determinant :: (m: Matrix4) -> float {
    d: float = 0;

    for j: 0..3 {
        term := m.coef[0][j] * minor4(m, 0, j);
        if (j & 1) == 0 d += term;
        else d -= term;
    }

    return d;
}

// The inverse from the adjugate and determinant. False and the identity when there is none.
//
// - m: the matrix.
// - epsilon: a determinant with a smaller magnitude counts as having no inverse.
inverse_lengyel :: (m: Matrix4, epsilon := 0.0001) -> (success: bool, result: Matrix4) {
    d := determinant(m);
    if abs(d) < epsilon return false, Matrix4_Identity;

    r := adjugate(m);
    for *x: r.floats x.* /= d;

    return true, r;
}

// The inverse of a matrix that is only a rotation and a translation.
//
// - m: the matrix. Anything else gives a wrong result.
isometry_inverse :: (m: Matrix4) -> Matrix4 {
    r := make_matrix4(transpose(make_matrix3(m)));
    t := -multiply(r, xyz(m._14, m._24, m._34));

    r._14 = t.x;
    r._24 = t.y;
    r._34 = t.z;

    return r;
}

// The largest sum of absolute values over the rows, or over the columns.
//
// - M: the matrix.
// - tpose: true sums down columns, false across rows.
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

// A vector `u` for a reflection that sends `v` onto the z axis. Zero gives zero.
//
// - v: the vector to reflect.
make_reflector :: (v: Vector3) -> (u: Vector3) {
    u := v;
    n := length(v);

    if n == 0 return .{};
    if v.z >= 0 u.z += n;
    else u.z -= n;

    return normalize(u) * cast(float) SQRT2;
}

// Reflects each of the matrix's row vectors `M.v[i]` in place.
//
// - M: the matrix to change.
// - u: the reflector, from `make_reflector`.
reflect_cols :: (M: *Matrix3, u: Vector3) {
    for i: 0..2 M.v[i] -= u * dot(M.v[i], u);
}

// Reflects each column of the matrix in place.
//
// - M: the matrix to change.
// - u: the reflector, from `make_reflector`.
reflect_rows :: (M: *Matrix3, u: Vector3) {
    r := transpose(M.*);
    reflect_cols(*r, u);
    M.* = transpose(r);
}

// Splits the matrix into an orthogonal `Q` and a symmetric `S` with `M = Q * S`, and also
// returns the determinant of `M`.
//
// - M: the matrix to split.
polar_decomp :: (M: Matrix3) -> (Q: Matrix3, S: Matrix3, det: float) {
    // Jacobi eigensystem of M^T M: singular values, then an orthogonal completion.
    OFF_DIAGONAL_EPSILON :: 0.00000001;
    SINGULAR_VALUE_EPSILON :: 0.0000001;

    c := multiply(transpose(M), M);
    v := Matrix3_Identity;

    for sweep: 0..31 {
        for p: 0..1 for q : p + 1..2 {
            a := c.coef[p][q];
            if abs(a) <= OFF_DIAGONAL_EPSILON * sqrt(abs(c.coef[p][p] * c.coef[q][q])) continue;

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
        if d.coef[i][i] > SINGULAR_VALUE_EPSILON invd.coef[i][i] = 1 / d.coef[i][i];
    }

    u := multiply(multiply(M, v), invd);

    // Fill null-space directions without disturbing nonzero singular vectors.
    for i: 0..2 if d.coef[i][i] <= SINGULAR_VALUE_EPSILON {
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
