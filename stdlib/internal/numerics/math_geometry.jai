// A plane from its coefficients, for the plane `a*x + b*y + c*z + d = 0`.
//
// - a: the normal's x.
// - b: the normal's y.
// - c: the normal's z.
// - d: the offset.
make_plane :: (a: float, b: float, c: float, d: float) -> Plane3 {
    return .{ a, b, c, d };
}

// A plane from a normal and an offset.
//
// - v: the normal.
// - d: the offset, added after the dot product with the normal.
make_plane :: (v: Vector3, d: float) -> Plane3 {
    return .{ v.x, v.y, v.z, d };
}

// A plane with the given normal that passes through a point.
//
// - v: the normal.
// - x: a point on the plane.
make_plane :: (v: Vector3, x: Vector3) -> Plane3 {
    return make_plane(v, -dot(v, x));
}

// A plane through a point with the given normal.
//
// - point: a point on the plane.
// - normal: the plane's normal.
plane_from_point_and_normal :: (point: Vector3, normal: Vector3) -> Plane3 {
    return make_plane(normal, point);
}

// A plane from a vector holding the normal in x, y, z and the offset in w.
//
// - v: the normal in x, y, z and the offset in w.
make_plane :: (v: Vector4) -> Plane3 {
    return .{ v.x, v.y, v.z, v.w };
}

// The plane's normal.
//
// - p: the plane.
get_normal :: (p: Plane3) -> Vector3 {
    return p.normal;
}

normalize :: (using p: Plane3) -> Plane3 {
    n := length(normal);
    if n == 0 return p;
    return .{ a / n, b / n, c / n, d / n };
} @UseResult

// The plane through three points, with a unit normal.
// The normal follows the winding of `p0`, `p1`, `p2`.
//
// - p0: the first point.
// - p1: the second point.
// - p2: the third point.
normalized_plane_from_points :: (p0: Vector3, p1: Vector3, p2: Vector3) -> Plane3 {
    return make_plane(unit_vector(cross(p1 - p0, p2 - p0)), p0);
}

// The signed distance from a point to the plane, if the normal has unit length.
// Positive on the side the normal faces.
//
// - plane: the plane.
// - v: the point.
plane_dot :: (plane: Plane3, v: Vector3) -> float #symmetric {
    return dot(plane.normal, v) + plane.d;
}

// A struct printer for matrices. Prints each row on its own line. Returns false for any other type.
//
// - builder: where the text is appended.
// - any: the value being printed.
// - user_data: unused.
print_matrix :: (builder: *String_Builder, any: Any, user_data: *void) -> bool {
    if any.type == type_info(Matrix2) {
        m := cast(*Matrix2) any.value_pointer;
        for r: m.v {
            print_components(builder, r.component);
            append(builder, "\n");
        }

        return true;
    }

    if any.type == type_info(Matrix3) {
        m := cast(*Matrix3) any.value_pointer;
        for r: m.v {
            print_components(builder, r.component);
            append(builder, "\n");
        }

        return true;
    }

    if any.type == type_info(Matrix4) {
        m := cast(*Matrix4) any.value_pointer;
        for r: m.v {
            print_components(builder, r.component);
            append(builder, "\n");
        }

        return true;
    }

    if any.type == type_info(Matrix4x3) {
        m := cast(*Matrix4x3) any.value_pointer;
        for r: m.v {
            print_components(builder, r.component);
            append(builder, "\n");
        }

        return true;
    }

    return false;
}

// A struct printer for the math types: matrices, vectors, quaternions and planes.
// Returns false for any other type.
//
// - builder: where the text is appended.
// - any: the value being printed.
// - user_data: unused.
math_struct_printer :: (builder: *String_Builder, any: Any, user_data: *void) -> bool {
    if print_matrix(builder, any, user_data) return true;

    if any.type == type_info(Vector2) {
        v := cast(*Vector2) any.value_pointer;
        print_components(builder, v.component);
        return true;
    }

    if any.type == type_info(Vector3) {
        v := cast(*Vector3) any.value_pointer;
        print_components(builder, v.component);
        return true;
    }

    if any.type == type_info(Vector4) {
        v := cast(*Vector4) any.value_pointer;
        print_components(builder, v.component);
        return true;
    }

    if any.type == type_info(Quaternion) {
        v := cast(*Quaternion) any.value_pointer;
        print_components(builder, v.component);
        return true;
    }

    if any.type == type_info(Plane3) {
        v := cast(*Plane3) any.value_pointer;
        print_components(builder, v.vector4.component);
        return true;
    }

    return false;
}

#scope_file
#import "Basic";
Num_Text :: #import "internal/numerics/number_text";

print_components :: (builder: *String_Builder, components: [] float) {
    append(builder, "(");

    for x, i: components {
        if i > 0 append(builder, ", ");
        Num_Text.append_number(builder, cast(float64) x, 6);
    }

    append(builder, ")");
}
