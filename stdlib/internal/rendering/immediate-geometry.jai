immediate_triangle :: (p0: Vector3, p1: Vector3, p2: Vector3, c1: Vector4, c2: Vector4, c3: Vector4, uv0 := Vector2.{}, uv1 := Vector2.{}, uv2 := Vector2.{}, normal := Vector3.{ z = 1 }) {
    immediate_vertex(p0, c1, uv0, normal);
    immediate_vertex(p1, c2, uv1, normal);
    immediate_vertex(p2, c3, uv2, normal);
}

immediate_quad :: (x0: float, y0: float, x1: float, y1: float, color: Vector4) {
    immediate_quad(Vector2.{ x0, y0 }, .{ x1, y0 }, .{ x1, y1 }, .{ x0, y1 }, color);
}

immediate_quad :: (p0: Vector2, p1: Vector2, p2: Vector2, p3: Vector2, color := Vector4.{ 1, 1, 1, 1 }, uv0 := Vector2.{ 0, 0 }, uv1 := Vector2.{ 1, 0 }, uv2 := Vector2.{ 1, 1 }, uv3 := Vector2.{ 0, 1 }) {
    immediate_quad(p0, p1, p2, p3, color, color, color, color, uv0, uv1, uv2, uv3);
}

immediate_quad :: (p0: Vector2, p1: Vector2, p2: Vector2, p3: Vector2, c0: Vector4, c1: Vector4, c2: Vector4, c3: Vector4, uv0 := Vector2.{ 0, 0 }, uv1 := Vector2.{ 1, 0 }, uv2 := Vector2.{ 1, 1 }, uv3 := Vector2.{ 0, 1 }) {
    a := Vector3.{ p0.x, p0.y, 0 };
    b := Vector3.{ p1.x, p1.y, 0 };
    c := Vector3.{ p2.x, p2.y, 0 };
    d := Vector3.{ p3.x, p3.y, 0 };

    immediate_triangle(a, b, c, c0, c1, c2, uv0, uv1, uv2);
    immediate_triangle(a, c, d, c0, c2, c3, uv0, uv2, uv3);
}

immediate_quad :: (p0: Vector3, p1: Vector3, p2: Vector3, p3: Vector3, color: Vector4, uv0 := Vector2.{ 0, 0 }, uv1 := Vector2.{ 1, 0 }, uv2 := Vector2.{ 1, 1 }, uv3 := Vector2.{ 0, 1 }, normal := Vector3.{ z = 1 }) {
    immediate_triangle(p0, p1, p2, color, color, color, uv0, uv1, uv2, normal);
    immediate_triangle(p0, p2, p3, color, color, color, uv0, uv2, uv3, normal);
}

immediate_quad :: (p0: Vector3, p1: Vector3, p2: Vector3, p3: Vector3, color: Vector4, uv0: Vector2, uv1: Vector2, uv2: Vector2, uv3: Vector2, st0: Vector2, st1: Vector2, st2: Vector2, st3: Vector2, normal: Vector3, tangent: Vector4) {
    full_vertex(p0, color, uv0, st0, normal, tangent);
    full_vertex(p1, color, uv1, st1, normal, tangent);
    full_vertex(p2, color, uv2, st2, normal, tangent);
    full_vertex(p0, color, uv0, st0, normal, tangent);
    full_vertex(p2, color, uv2, st2, normal, tangent);
    full_vertex(p3, color, uv3, st3, normal, tangent);
}

#scope_module
full_vertex :: (position: Vector3, color: Vector4, uv: Vector2, st: Vector2, normal: Vector3, tangent: Vector4) {
    immediate_vertex(position, color, uv, normal);
    vertex := *immediate_state.vertices[immediate_state.num_vertices - 1];
    vertex.uv1 = st;
    vertex.tangent = tangent;
}
