#import "Basic";
#import "Math";
#import "Sound_Player";

near :: (a: float, b: float) -> bool {
    return abs(a - b) < 0.00001;
}

main :: () {
    pcm := s16.[1000, 2000, 3000];
    data: Sound_Data;
    data.loaded = true;
    data.type = .LINEAR_SAMPLE_ARRAY;
    data.samples = pcm.data;
    data.nsamples_times_nchannels = pcm.count;
    stream: Sound_Stream;
    stream.sound_data = *data;
    stream.num_channels = 1;
    stream.repeat_end_position = 2;
    bounds: Buffer_Bounds;
    found, safe := get_buffer_bounds(*stream, *data, 1, *bounds);

    assert(found && safe && bounds.start == 0 && bounds.end == 2);
    sample: [1] float;

    sample_n_channels(*stream, pcm.data, 0, 2, 1.5, sample, safe);
    assert(near(sample[0], 2000.0 / 32768.0), "final page sample is held");

    output: [4] Audio_Dest_Sample;

    stream.input_scale_mappings[0].source_scale_for_this_output_index[0] = 1;
    stream.input_scale_mappings[0].interpolated_source_scale_for_this_output_index[0] = 1;
    stream.play_cursor = 2;
    assert(!accumulate_stream(*stream, output, 2));
    assert(stream.play_cursor == 4 && output[0].channels[0] == 0);
    stream.repeat_end_position = -1;
    stream.play_cursor = 1;
    stream.current_rate = 0;
    stream.desired_rate = 0;
    assert(!accumulate_stream(*stream, output, 2));
    assert(stream.play_cursor == 1 && near(output[0].channels[0], 2000.0 / 32768.0));
    memset(output.data, 0, size_of(output));
    stream.play_cursor = 2;
    stream.current_rate = -1;
    stream.desired_rate = -1;
    assert(!accumulate_stream(*stream, output, 2));
    assert(stream.play_cursor == 0);
    assert(near(output[0].channels[0], 3000.0 / 32768.0));
    assert(near(output[1].channels[0], 2000.0 / 32768.0));
    stream.rate_scale = -2;
    update_desired_rate(*stream);
    assert(stream.desired_rate == -2);
    memset(output.data, 0, size_of(output));
    data.type = .OGG_COMPRESSED;
    stream.current_rate = 1;
    stream.desired_rate = 1;
    stream.play_cursor = 0;
    assert(!accumulate_stream(*stream, output, 2));
    assert(stream.play_cursor == 2 && output[0].channels[0] == 0, "missing pages advance through silence");
    stream.internal_flags |= .WAITING_FOR_INITIAL_DECODER_PAGES;
    assert(!accumulate_stream(*stream, output, 2));
    assert(stream.play_cursor == 2, "initial decoder wait holds the cursor");

    #if OS == .MACOS || OS == .WINDOWS {
        assert(get_bytes_between(cast(s64) 2, 8, 10) == 4);
        assert(get_bytes_between(cast(u32) 8, 2, 10) == 6);
        assert(get_bytes_between(cast(s32) 3, 3, 10) == 0);
    }

    #if OS == .LINUX assert(!backend_needs_async_update_from_main_thread());
    else assert(backend_needs_async_update_from_main_thread());
    stream.user_flags = .{};
    assert(!skip_this_output_channel(.SUBWOOFER, *stream));
    stream.user_flags |= .IS_FOOTSTEP;
    assert(skip_this_output_channel(.REAR_LEFT, *stream));
    assert(skip_this_output_channel(.REAR_RIGHT, *stream));
    assert(!skip_this_output_channel(.FRONT_LEFT, *stream));
    assert(!skip_this_output_channel(.SUBWOOFER, *stream));

    // No listener/nonspatial sources use the locked plane.
    stream.position = .{ 0, 5, 0 };
    direction := get_plane_dir(*stream, stream.position);

    assert(direction.x == 1 && direction.y == 0);
    update_listener(.{}, Quaternion.{});
    direction = get_plane_dir(*stream, stream.position);
    assert(direction.x == 1 && direction.y == 0);
    stream.user_flags |= .SPATIALIZED;
    stream.inner_radius = 4;
    stream.position = .{ 0, 1.5, 100 };
    assert(near(get_target_2_closeness(*stream), 0.5), "closeness uses planar distance and inner radius");
    left: float = 0;
    right: float = 0.5;

    adjust_stereo_dots_by_closeness(*left, *right, 0.5);
    assert(near(left, 0.5) && near(right, 0.75));

    arcs = .{};
    scale: Scale_Mapping;

    add_vbap_gain_for_direction(*scale, .{ 1, 0 }, 1);
    for scale.source_scale_for_this_output_index assert(it == 0);

    vbap_speaker_directions[0] = .{ 1, 0 };
    vbap_speaker_directions[1] = .{ 0, 1 };
    pair := Vbap_Arc.[.{ .FRONT_LEFT, .FRONT_RIGHT, .{} }];

    arcs = pair;
    add_vbap_gain_for_direction(*scale, .{ -1, 1 }, 1);
    assert(scale.source_scale_for_this_output_index[0] == 0 && scale.source_scale_for_this_output_index[1] > 0, "outside coverage retains positive selected component");
    vbap_speaker_directions[2] = .{ -ROOT_2_OVER_2, ROOT_2_OVER_2 };
    overlapping := Vbap_Arc.[.{ .FRONT_LEFT, .FRONT_RIGHT, .{} }, .{ .FRONT_LEFT, .CENTER, .{} }];

    arcs = overlapping;
    scale = .{};
    add_vbap_gain_for_direction(*scale, .{ 0.1, 1 }, 1);
    assert(scale.source_scale_for_this_output_index[2] > 0 && scale.source_scale_for_this_output_index[1] == 0, "select greatest minimum, not first covering arc");
    scale = .{};
    add_vbap_gain_for_direction(*scale, .{ -1, 0.2 }, 1);
    assert(scale.source_scale_for_this_output_index[2] > 0 && scale.source_scale_for_this_output_index[1] == 0, "outside coverage still selects greatest minimum");
    channel, success := source_dir_to_target_dir(.MONO);

    assert(success && channel == .CENTER);
    channel, success = source_dir_to_target_dir(.EXTEND_LEFT);
    assert(success && channel == .REAR_LEFT);
    print("ok\n");
}
