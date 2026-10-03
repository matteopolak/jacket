make_plane :: (a: float, b: float, c: float, d: float) -> Plane3 {
    return .{ a, b, c, d };
}

make_plane :: (v: Vector3, d: float) -> Plane3 {
    return .{ v.x, v.y, v.z, d };
}

make_plane :: (v: Vector3, x: Vector3) -> Plane3 {
    return make_plane(v, -dot(v, x));
}

plane_from_point_and_normal :: (point: Vector3, normal: Vector3) -> Plane3 {
    return make_plane(normal, point);
}

make_plane :: (v: Vector4) -> Plane3 {
    return .{ v.x, v.y, v.z, v.w };
}

get_normal :: (p: Plane3) -> Vector3 {
    return p.normal;
}

normalize :: (using p: Plane3) -> Plane3 {
    n := length(normal);
    if n == 0 return p;
    return .{ a / n, b / n, c / n, d / n };
}

normalized_plane_from_points :: (p0: Vector3, p1: Vector3, p2: Vector3) -> Plane3 {
    return make_plane(unit_vector(cross(p1 - p0, p2 - p0)), p0);
}

plane_dot :: (plane: Plane3, v: Vector3) -> float #symmetric {
    return dot(plane.normal, v) + plane.d;
}

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
