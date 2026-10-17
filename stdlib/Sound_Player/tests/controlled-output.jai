#import "Basic";
#import "Math";
#import "Sound_Player";

main :: () {
    config: Sound_Player_Config;
    config.update_from_a_thread = false;
    set_master_volume(0);

    if !sound_player_init(config) {
        sound_player_shutdown();
        print("native-device-unavailable\nok\n");
        return;
    }

    properties := backend_init(config);

    #if #exists(EXPECTED_CHANNELS) assert(properties.num_channels == EXPECTED_CHANNELS);
    print("native-%-channel-output-opened\n", properties.num_channels);
    directions := Source_Direction.[.MONO, .LEFT, .RIGHT, .FRONT_LEFT, .FRONT_RIGHT, .REAR_LEFT, .REAR_RIGHT, .CENTER_FORWARD, .SUBWOOFER_CENTER_FORWARD, .EXTEND_LEFT, .EXTEND_RIGHT];
    expected_channels := Audio_Channel.[.CENTER, .FRONT_LEFT, .FRONT_RIGHT, .FRONT_LEFT, .FRONT_RIGHT, .REAR_LEFT, .REAR_RIGHT, .CENTER, .SUBWOOFER, .REAR_LEFT, .REAR_RIGHT];

    if properties.num_channels == 8 {
        expected_channels[9] = .EXTEND_LEFT;
        expected_channels[10] = .EXTEND_RIGHT;
    }

    for direction: directions {
        channel, success := source_dir_to_target_dir(direction);
        assert(success && channel == expected_channels[it_index]);
    }

    // VBAP requires an initialized multichannel output, not the stereo fallback.
    // Preserve the exact pair-direction regression without exposing private helpers.
    if properties.num_channels >= 6 {
        probe: Sound_Stream;
        probe.position = .{ 0, 10, 0 };
        mapping: Scale_Mapping;

        accum_scale_target_all_source_mono(*probe, vbap_speaker_directions[cast(s64) Audio_Channel.FRONT_LEFT], 1, *mapping);
        assert(abs(mapping.source_scale_for_this_output_index[0] - 1) < 0.0001);
        for i: 1..properties.num_channels - 1 assert(abs(mapping.source_scale_for_this_output_index[i]) < 0.0001);
    }

    // Native output already holds only silence. Pull all nonzero test samples
    // directly into this controlled buffer; never submit them to the device.
    pcm := s16.[24576];
    data: Sound_Data;
    data.loaded = true;
    data.type = .LINEAR_SAMPLE_ARRAY;
    data.nchannels = 1;
    data.samples = pcm.data;
    data.nsamples_times_nchannels = 1;
    // Reviewed locked and music mappings, without starting these probe records.
    probe: Sound_Stream;
    probe.sound_data = *data;
    probe.num_channels = 1;
    update_panning_helper_for_many_channel_music(*probe, 1);

    if properties.num_channels == 2 {
        assert(probe.input_scale_mappings[0].source_scale_for_this_output_index[0] == 0.5);
        assert(probe.input_scale_mappings[0].source_scale_for_this_output_index[1] == 0.5);
    } else {
        scales := probe.input_scale_mappings[0].source_scale_for_this_output_index;
        assert(scales[0] == 0.8 && scales[1] == 0.8 && scales[2] == 1 && scales[3] == 1);
        for i: 4..properties.num_channels - 1 assert(scales[i] == 0.5);
    }

    probe.input_scale_mappings = .{};
    probe.num_channels = 2;
    update_panning_helper_for_many_channel_music(*probe, 1);
    assert(probe.input_scale_mappings[0].source_scale_for_this_output_index[0] == 1);
    assert(probe.input_scale_mappings[1].source_scale_for_this_output_index[1] == 1);
    assert(probe.input_scale_mappings[0].source_scale_for_this_output_index[1] == 0);

    if properties.num_channels > 2 {
        assert(probe.input_scale_mappings[0].source_scale_for_this_output_index[2] == 0.7);
        assert(probe.input_scale_mappings[1].source_scale_for_this_output_index[3] == 0.7);
        assert(probe.input_scale_mappings[0].source_scale_for_this_output_index[4] == 0.5);
        probe.input_scale_mappings = .{};
        probe.num_channels = 1;
        probe.user_flags = .SPATIALIZED | .IS_FOOTSTEP;
        probe.inner_radius = 4;
        update_listener(.{}, Quaternion.{});
        scale: Scale_Mapping;

        accum_scale_target_all_source_mono(*probe, .{ 1, 0 }, 1, *scale);
        assert(scale.source_scale_for_this_output_index[3] == 0);
        assert(scale.source_scale_for_this_output_index[4] == 0 && scale.source_scale_for_this_output_index[5] == 0);
        power: float;
        for i: 0..properties.num_channels - 1 power += scale.source_scale_for_this_output_index[i] * scale.source_scale_for_this_output_index[i];

        assert(abs(power - 1) < 0.00001);
    }

    set_master_volume(1);
    a := make_stream(*data);
    b := make_stream(*data);
    c := make_stream(*data);

    set_repeating(a, true);
    set_repeating(b, true);
    set_repeating(c, true);
    start_playing(a);
    start_playing(b);
    start_playing(c);
    output: [32] s16;
    bytes: string = .{ properties.num_channels * 2 * 4, cast(*u8) output.data };

    fill_sample_buffer(bytes);

    if properties.num_channels == 2 {
        for i: 0..7 assert(output[i] == 32767, "positive stereo clipping at %: %", i, output[i]);
    } else {
        for frame: 0..3 assert(output[frame * properties.num_channels + 2] == 32767, "positive center clipping");
    }

    b.user_flags |= .PAUSED_DUE_TO_MENU;
    c.user_flags |= .PAUSED_DUE_TO_MENU;
    before := b.play_cursor;

    fill_sample_buffer(bytes);
    assert(b.play_cursor == before);
    if properties.num_channels == 2 assert(output[0] == 12288);
    else assert(output[2] == 24576);

    b.user_flags &= ~ .PAUSED_DUE_TO_MENU;
    c.user_flags &= ~ .PAUSED_DUE_TO_MENU;
    pcm[0] = -24576;
    fill_sample_buffer(bytes);

    if properties.num_channels == 2 {
        for i: 0..7 assert(output[i] == -32768, "negative stereo clipping");
    } else {
        for frame: 0..3 assert(output[frame * properties.num_channels + 2] == -32768, "negative center clipping");
    }

    set_master_volume(0);
    update_panning(a);
    gain_channel := 0;
    if properties.num_channels > 2 gain_channel = 2;

    assert(a.input_scale_mappings[0].dvolume_dsample[gain_channel] < 0, "fade derivative must retain its sign");
    stop_stream_abruptly(*data);
    sound_player_shutdown();
    print("ok\n");
}
