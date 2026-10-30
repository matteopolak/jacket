operator == :: (a: Quaternion, b: Quaternion) -> bool {
    return a.x == b.x && a.y == b.y && a.z == b.z && a.w == b.w;
}

operator + :: (a: Quaternion, b: Quaternion) -> Quaternion {
    return .{ a.x + b.x, a.y + b.y, a.z + b.z, a.w + b.w };
}

operator - :: (a: Quaternion, b: Quaternion) -> Quaternion {
    return .{ a.x - b.x, a.y - b.y, a.z - b.z, a.w - b.w };
}

operator - :: (a: Quaternion) -> Quaternion {
    return .{ -a.x, -a.y, -a.z, -a.w };
}

operator * :: (a: Quaternion, k: float) -> Quaternion #symmetric {
    return .{ a.x * k, a.y * k, a.z * k, a.w * k };
}

// The Hamilton product: `a * b` applies `b`'s rotation first, then `a`'s.
operator * :: (a: Quaternion, b: Quaternion) -> Quaternion {
    vector := a.w * b.xyz + b.w * a.xyz + cross(a.xyz, b.xyz);
    return .{ vector.x, vector.y, vector.z, a.w * b.w - dot_product(a.xyz, b.xyz) };
}

dot_product :: (a: Quaternion, b: Quaternion) -> float {
    return a.x * b.x + a.y * b.y + a.z * b.z + a.w * b.w;
} @UseResult

length :: (a: Quaternion) -> float {
    return length(xyzw(a.x, a.y, a.z, a.w));
} @UseResult

// Scales `*q` to length 1 in place and returns its old length. A zero quaternion becomes the
// identity.
//
// - q: the quaternion to normalize.
normalize_or_identity :: (using q: *Quaternion) -> float {
    wide := length_wide(q.*);
    n := cast(float) wide;

    if wide == 0 q.* = .{};
    else {
        for *coefficient: q.component coefficient.* = cast(float) (cast(float64) coefficient.* / wide);
    }

    return n;
}

lerp :: (a: Quaternion, b: Quaternion, t: float) -> Quaternion {
    return .{ lerp(a.x, b.x, t), lerp(a.y, b.y, t), lerp(a.z, b.z, t), lerp(a.w, b.w, t) };
} @UseResult

// Interpolates linearly, then normalizes the result.
//
// - p0: the quaternion at `fraction` 0.
// - p1: the quaternion at `fraction` 1.
// - fraction: how far from `p0` to `p1`, from 0 to 1.
nlerp :: inline (p0: Quaternion, p1: Quaternion, fraction: float) -> Quaternion {
    q := lerp(p0, p1, fraction);
    normalize_or_identity(*q);
    return q;
}

// Interpolates along the shorter arc at constant angular speed. Falls back to `nlerp` when the two
// are nearly parallel.
//
// - start: the quaternion at `t` 0.
// - end: the quaternion at `t` 1.
// - t: how far from `start` to `end`, from 0 to 1.
slerp :: (start: Quaternion, end: Quaternion, t: float) -> Quaternion {
    d := dot_product(start, end);
    e := end;

    if d < 0 {
        d = -d;
        e = .{ -e.x, -e.y, -e.z, -e.w };
    }

    // Close enough to parallel that the sine in the slerp weights would be nearly zero.
    NEARLY_PARALLEL :: 0.9995;

    if d > NEARLY_PARALLEL return nlerp(start, e, t);

    a := acos(clamp(d, -1.0, 1.0));
    s := sin(a);
    u := sin((1 - t) * a) / s;
    v := sin(t * a) / s;

    return .{ start.x * u + e.x * v, start.y * u + e.y * v, start.z * u + e.z * v, start.w * u + e.w * v };
}

// Sets `*q` to a rotation about an axis. The axis is not normalized, so pass a unit vector.
//
// - q: the quaternion to write.
// - v: the rotation axis.
// - theta: the angle in radians.
set_from_axis_and_angle :: (using q: *Quaternion, v: Vector3, theta: float) {
    s := sin(theta / 2);

    x = v.x * s;
    y = v.y * s;
    z = v.z * s;
    w = cos(theta / 2);
}

// Sets `*q` to a rotation about an axis. The axis is not normalized, so pass a unit vector.
//
// - q: the quaternion to write.
// - x: the x of the rotation axis.
// - y: the y of the rotation axis.
// - z: the z of the rotation axis.
// - theta: the angle in radians.
set_from_axis_and_angle :: inline (q: *Quaternion, x: float, y: float, z: float, theta: float) {
    set_from_axis_and_angle(q, .{ x, y, z }, theta);
}

// Returns the unit axis and the angle in radians of the rotation. A quaternion with no rotation
// part gives the axis (1, 0, 0) and angle 0.
//
// - q: the rotation to read.
get_axis_and_angle :: (using q: Quaternion) -> x: float64, y: float64, z: float64, theta: float64 {
    s := sqrt(cast(float64) (q.x * q.x + q.y * q.y + q.z * q.z));
    if s == 0 return 1, 0, 0, 0;
    return q.x / s, q.y / s, q.z / s, 2 * atan2(s, cast(float64) q.w);
}

// Negates the vector part. For a unit quaternion this is the inverse rotation.
//
// - q: the quaternion to conjugate.
conjugate :: (using q: Quaternion) -> Quaternion {
    return .{ -x, -y, -z, w };
}

rotate :: (v: Vector3, q: Quaternion) -> Vector3 {
    u := q.xyz;
    t := 2 * cross(u, v);
    return v + q.w * t + cross(u, t);
}

rotate :: (v: *Vector3, q: Quaternion) {
    v.* = rotate(v.*, q);
}

// Writes the rotation into the upper-left 3x3 of `*m`. Other entries are left alone.
//
// - m: the matrix to write.
// - q: the rotation, expected to be a unit quaternion.
set_rotation :: (using m: *$T/interface AnyMatrix3, q: Quaternion) {
    x := q.x;
    y := q.y;
    z := q.z;
    w := q.w;

    _11 = 1 - 2 * (y * y + z * z);
    _12 = 2 * (x * y - z * w);
    _13 = 2 * (x * z + y * w);
    _21 = 2 * (x * y + z * w);
    _22 = 1 - 2 * (x * x + z * z);
    _23 = 2 * (y * z - x * w);
    _31 = 2 * (x * z - y * w);
    _32 = 2 * (y * z + x * w);
    _33 = 1 - 2 * (x * x + y * y);
}

// A rotation matrix of type `T`: the identity with the rotation written in.
//
// - $T: the matrix type.
// - q: the rotation, expected to be a unit quaternion.
rotation_matrix :: ($T: Type, q: Quaternion) -> T {
    m := identity_of(T);
    set_rotation(*m, q);
    return m;
}

// The quaternion for the rotation in the upper-left 3x3 of `m`. The matrix is expected to be a pure
// rotation.
//
// - m: the matrix to read.
get_rotation :: (using m: $T/interface AnyMatrix3) -> Quaternion {
    q: Quaternion;
    tr := _11 + _22 + _33;

    if tr > 0 {
        s := 2 * sqrt(tr + 1);

        q.w = s / 4;
        q.x = (_32 - _23) / s;
        q.y = (_13 - _31) / s;
        q.z = (_21 - _12) / s;
    } else if _11 > _22 && _11 > _33 {
        s := 2 * sqrt(1 + _11 - _22 - _33);

        q.x = s / 4;
        q.y = (_12 + _21) / s;
        q.z = (_13 + _31) / s;
        q.w = (_32 - _23) / s;
    } else if _22 > _33 {
        s := 2 * sqrt(1 + _22 - _11 - _33);

        q.y = s / 4;
        q.x = (_12 + _21) / s;
        q.z = (_23 + _32) / s;
        q.w = (_13 - _31) / s;
    } else {
        s := 2 * sqrt(1 + _33 - _11 - _22);

        q.z = s / 4;
        q.x = (_13 + _31) / s;
        q.y = (_23 + _32) / s;
        q.w = (_21 - _12) / s;
    }

    return q;
}

// Builds two unit axes perpendicular to `x_axis` and to each other. Picks a reference axis itself.
//
// - x_axis: the first axis of the basis.
make_an_orthonormal_basis :: (x_axis: Vector3) -> (y_axis: Vector3, z_axis: Vector3) {
    approximate := Vector3.{ 0, 0, 1 };
    NEARLY_ALONG_Z :: 0.9;

    if abs(x_axis.z) > NEARLY_ALONG_Z approximate = .{ 0, 1, 0 };
    return make_an_orthonormal_basis(x_axis, approximate);
}

// Builds two unit axes perpendicular to `x_axis` and to each other. The returned z axis is
// perpendicular to `approximate_axis` too, so that axis steers which way the basis points.
//
// - x_axis: the first axis of the basis.
// - approximate_axis: a rough direction for the y axis. If parallel to `x_axis`, another reference
//   is used.
make_an_orthonormal_basis :: (x_axis: Vector3, approximate_axis: Vector3) -> (y_axis: Vector3, z_axis: Vector3) {
    x := unit_vector(x_axis);
    z := unit_vector(cross(x, approximate_axis));

    if length_squared(z) == 0 {
        a := Vector3.{ 1, 0, 0 };
        NEARLY_ALONG_X :: 0.9;

        if abs(x.x) > NEARLY_ALONG_X a = .{ 0, 1, 0 };
        z = unit_vector(cross(x, a));
    }

    return cross(z, x), z;
}

#scope_file
#import "Basic";
