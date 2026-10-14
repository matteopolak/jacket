#import "Basic";
#import "Math";
C :: #import "Color_Interpolation";
S :: #import "Sloppy_Math";
G :: #import "GetRect";

main :: () {
    assert(S.min_of_type(float32) == FLOAT32_MIN);
    assert(S.min_of_type(s16) == -32768 && S.max_of_type(u64) == 0xffff_ffff_ffff_ffff);
    q: Quaternion;

    set_from_axis_and_angle(*q, xyz(0, 0, 1), PI / 2);
    v := rotate(xyz(1, 0, 0), q);

    assert(abs(v.x) < 0.000001 && abs(v.y - 1) < 0.000001);
    r := rotation_matrix(Matrix4, q);
    recovered := get_rotation(r);

    assert(abs(abs(dot(q, recovered)) - 1) < 0.000001);
    m := multiply(make_translation_matrix4(xyz(3, 4, 5)), make_scale_matrix4(xyz(2, 3, 4)));
    ok, inv := inverse(m);

    assert(ok);
    product := multiply(m, inv);
    for i: 0..15 assert(abs(product.floats[i] - Matrix4_Identity.floats[i]) < 0.000001);

    assert(abs(determinant(m) - 24) < 0.00001);
    ok, inv = inverse_lengyel(m);
    assert(ok);
    product = multiply(inv, m);
    for i: 0..15 assert(abs(product.floats[i] - Matrix4_Identity.floats[i]) < 0.000001);

    a := Matrix3.{ 2, 1, 0, 0, 3, 1, 0, 0, 4 };
    Q, S, det := polar_decomp(a);
    reconstructed := multiply(Q, S);

    for i: 0..8 assert(abs(a.floats[i] - reconstructed.floats[i]) < 0.0001);

    orthogonal := multiply(transpose(Q), Q);
    for i: 0..8 assert(abs(orthogonal.floats[i] - Matrix3_Identity.floats[i]) < 0.0001);

    assert(abs(det - 24) < 0.001);
    projection := make_projection_matrix(PI / 2, 1, 1, 10);
    clip := multiply(projection, xyzw(0, 0, -1, 1));

    assert(abs(clip.z / clip.w + 1) < 0.000001);
    plane := normalized_plane_from_points(xyz(0), xyz(1, 0, 0), xyz(0, 1, 0));

    assert(abs(plane_dot(plane, xyz(0, 0, 2)) - 2) < 0.00001);
    k0 := C.Color_Keyframe.{ .{ 0, 0, 0, 0 }, 0 };
    k1 := C.Color_Keyframe.{ .{ 1, 1, 1, 1 }, 1 };
    keys: [2] *C.Color_Keyframe = .[*k0, *k1];
    samples: [3] Vector4;

    C.generate_color_samples(keys, .{ .RGB, .LINEAR, false }, samples);
    assert(samples[0].x == 0 && samples[2].x == 1 && abs(cast(float64) samples[1].x - 0.7297400528) < 0.000001 && samples[1].w == 0.5);
    k0.color_rgb = xyzw(C.hsv_to_rgb(xyz(350, 1, 1)), 1);
    k1.color_rgb = xyzw(C.hsv_to_rgb(xyz(10, 1, 1)), 1);
    C.generate_color_samples(keys, .{ .HSV, .LINEAR, false }, samples);
    assert(abs(samples[1].x - 1) < 0.000001 && samples[1].y < 0.000001 && samples[1].z < 0.000001);
    k0.t = 0.25;
    k1.t = 0.75;
    C.generate_color_samples(keys, .{ .HSV, .LINEAR, true }, samples);
    assert(abs(samples[0].x - 1) < 0.000001 && samples[0].y < 0.000001);
    empty: [] *C.Color_Keyframe;
    before := samples[0];

    C.generate_color_samples(empty, .{}, samples);
    assert(samples[0].x == before.x);
    one: [1] Vector4;

    C.generate_color_samples(keys, .{}, one);
    assert(one[0].x == k0.color_rgb.x && one[0].w == k0.color_rgb.w);
    t: G.Text_Input_State;

    G.set_input(*t, "héllo world");
    G.move_cursor_to_end(*t);
    G.move_cursor_left_by_word(*t);
    assert(t.insert_point == 7);
    G.backspace_from_cursor(*t);
    assert(t.text == "hélloworld");
    G.select_all(*t);
    assert(G.insert_text(*t, "replacement"));
    assert(t.text == "replacement");
    G.move_cursor_to_beginning(*t);
    G.delete_from_cursor(*t);
    assert(t.text == "eplacement");
    G.set_text(*t, "hé");
    G.move_cursor_to_end(*t);
    G.backspace_from_cursor(*t);
    assert(t.text == "h");
    st: G.Color_Picker_State;

    G.set_current_color_hsv(*st, xyz(120, 1, 1));
    assert(st.current_rgb_color.y == 1);
    editor: G.Color_Animation_Editor_State;
    frames: [2] C.Color_Keyframe = .[k1, k0];

    G.set_keyframes(*editor, frames);
    sorted := G.get_sorted_keyframes(*editor);

    assert(sorted[0].t == k0.t && sorted[1].t == k1.t);
    array_free(sorted);
    interp := C.Interpolation.{ .HSV, .HERMITE, true };

    G.set_interpolation(*editor, interp);
    back := G.get_interpolation(*editor);

    assert(back.color_space == interp.color_space && back.wrapping);
    print("extended numerics ok\n");
}
