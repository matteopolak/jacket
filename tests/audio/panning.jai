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
    stream: Sound_Stream;
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
    assert(near(scale.source_scale_for_this_output_index[0], 1));
    assert(near(scale.source_scale_for_this_output_index[1], 0));
    channel, success := source_dir_to_target_dir(.SUBWOOFER_CENTER_FORWARD);

    assert(success && channel == .SUBWOOFER);
    channel, success = source_dir_to_target_dir(.MONO);
    assert(!success);

    arcs = arcs_6ch;
    scale = .{};
    accum_scale_target_all_source_mono(*stream, vbap_speaker_directions[cast(int) Audio_Channel.FRONT_LEFT], 1, *scale);
    assert(near(scale.source_scale_for_this_output_index[0], 1));
    original := arcs_6ch[0];

    arcs_6ch[0].speaker1 = .SUBWOOFER;
    assert(arcs[0].speaker1 == .SUBWOOFER);
    arcs_6ch[0] = original;
    print("ok\n");
}
