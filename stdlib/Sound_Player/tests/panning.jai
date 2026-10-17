#import "Basic";
#import "Math";
#import "Sound_Player";

near :: (a: float, b: float) -> bool {
    return abs(a - b) < 0.0001;
}

main :: () {
    m: Matrix2 = .{ 2, 1, 0, 4 };
    inverse: Matrix2;

    invert(*m, *inverse);
    assert(near(inverse._11, 0.5) && near(inverse._12, -0.125));
    invert(*m, *m);
    assert(near(m._11, inverse._11) && near(m._22, inverse._22));
    // Uniform scale changes determinant magnitude, not invertibility or condition.
    tiny: Matrix2 = .{ 0.0001, 0, 0, 0.0001 };

    invert(*tiny, *inverse);
    assert(near(tiny._11 * inverse._11, 1) && near(tiny._22 * inverse._22, 1));
    stream: Sound_Stream;
    stream.user_flags |= .SPATIALIZED;
    stream.position = .{ 0, 10, 0 };
    update_listener(.{}, Quaternion.{});
    direction := get_plane_dir(*stream, stream.position);

    assert(near(direction.x, 0) && near(direction.y, 1));
    turn: Quaternion = .{ 0, 0, 1, 0 };

    update_listener(.{}, turn);
    direction = get_plane_dir(*stream, stream.position);
    assert(near(direction.x, 0) && near(direction.y, -1));
    update_listener(.{}, Quaternion.{});
    scale: Scale_Mapping;

    accum_scale_target_2_source_mono(*stream, .{ 1, 0 }, 1, *scale);
    assert(near(scale.source_scale_for_this_output_index[0], sqrt(0.5)));
    assert(near(scale.source_scale_for_this_output_index[1], sqrt(0.5)));
    scale = .{};
    stream.position = .{ 0, 10, 0 };
    accum_scale_target_2_source_mono(*stream, .{ 0, 1 }, 1, *scale);
    left := scale.source_scale_for_this_output_index[0];
    right := scale.source_scale_for_this_output_index[1];

    assert(left > right && right > 0);
    assert(near(left * left + right * right, 1));
    channel, success := source_dir_to_target_dir(.SUBWOOFER_CENTER_FORWARD);

    assert(success && channel == .SUBWOOFER);
    channel, success = source_dir_to_target_dir(.MONO);
    assert(success && channel == .CENTER);

    vbap_speaker_directions = .[
        .{ ROOT_2_OVER_2, ROOT_2_OVER_2 }, .{ ROOT_2_OVER_2, -ROOT_2_OVER_2 },
        .{ 1, 0 }, .{ 0, 0 }, .{ -ROOT_2_OVER_2, ROOT_2_OVER_2 },
        .{ -ROOT_2_OVER_2, -ROOT_2_OVER_2 }, .{ -0.9659258, 0.2588190 }, .{ -0.9659258, -0.2588190 }
    ];
    arcs = arcs_6ch;
    original := arcs_6ch[0];

    arcs_6ch[0].speaker1 = .SUBWOOFER;
    assert(arcs[0].speaker1 == .SUBWOOFER);
    arcs_6ch[0] = original;
    print("ok\n");
}
