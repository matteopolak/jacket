#import "Basic";
#import "Sound_Player";

main :: () {
    pcm := s16.[0, 16384, -16384, 32767];
    data: Sound_Data;
    data.loaded = true;
    data.type = .LINEAR_SAMPLE_ARRAY;
    data.nchannels = 1;
    data.nsamples_times_nchannels = 4;
    data.samples = pcm.data;
    stream: Sound_Stream;
    stream.sound_data = *data;
    stream.num_channels = 1;
    stream.current_rate = 0.5;
    stream.desired_rate = 0.5;
    stream.input_scale_mappings[0].source_scale_for_this_output_index[0] = 1;
    stream.input_scale_mappings[0].interpolated_source_scale_for_this_output_index[0] = 1;
    bounds: Buffer_Bounds;
    found, safe := get_buffer_bounds(*stream, *data, 1, *bounds);

    assert(found && safe && bounds.start == 0 && bounds.end == 4 && bounds.memory == pcm.data);
    found, safe = get_buffer_bounds(*stream, *data, 4, *bounds);
    assert(!found && !safe);
    output: [9] Audio_Dest_Sample;
    finished := accumulate_stream(*stream, output, 9);

    assert(!finished && stream.play_cursor == 4.5);
    assert(output[0].channels[0] == 0);
    assert(output[1].channels[0] == 0.25);
    assert(output[2].channels[0] == 0.5);
    assert(output[3].channels[0] == 0);
    assert(output[4].channels[0] == -0.5);
    assert(output[8].channels[0] == 0);

    stream.user_flags |= .REPEATING;
    stream.repeat_start_position = 1;
    stream.repeat_end_position = 3;
    stream.play_cursor = 5.25;
    maybe_wrap_play_cursor(*stream);
    assert(stream.play_cursor == 1.25);
    stream.play_cursor = 1;
    stream.current_rate = 1;
    stream.desired_rate = 1;
    stream.silent_samples_at_start_of_play = 2;
    repeated: [6] Audio_Dest_Sample;

    finished = accumulate_stream(*stream, repeated, 6);
    assert(!finished);
    assert(repeated[0].channels[0] == 0 && repeated[1].channels[0] == 0);
    assert(repeated[2].channels[0] == 0.5 && repeated[3].channels[0] == -0.5);
    assert(repeated[4].channels[0] == 0.5 && repeated[5].channels[0] == -0.5);
    assert(stream.play_cursor == 3);
    print("ok\n");
}
