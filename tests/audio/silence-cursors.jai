#import "Basic";
#import "Sound_Player";

main :: () {
    pcm := s16.[8192, 16384, -8192, -16384];
    data: Sound_Data;
    data.type = .LINEAR_SAMPLE_ARRAY;
    data.nchannels = 1;
    data.nsamples_times_nchannels = pcm.count;
    data.samples = pcm.data;
    stream: Sound_Stream;
    stream.sound_data = *data;
    stream.num_channels = 1;
    stream.play_cursor = -2;
    stream.input_scale_mappings[0].interpolated_source_scale_for_this_output_index[0] = 1;
    stream.input_scale_mappings[0].source_scale_for_this_output_index[0] = 1;
    output: [8] Audio_Dest_Sample;
    finished := accumulate_stream(*stream, output, output.count);

    assert(!finished && stream.play_cursor == 6);
    assert(output[0].channels[0] == 0 && output[1].channels[0] == 0);
    assert(output[2].channels[0] == 0.25 && output[3].channels[0] == 0.5);
    assert(output[6].channels[0] == 0 && output[7].channels[0] == 0);
    stream.play_cursor = 0;
    stream.desired_rate = 2;
    stream.user_flags |= .PAUSED_DUE_TO_MENU;
    accumulate_stream(*stream, output, 1);
    assert(stream.play_cursor == 0);
    stream.user_flags &= ~ .PAUSED_DUE_TO_MENU;
    accumulate_stream(*stream, output, 1);
    assert(stream.play_cursor == 2 && stream.current_rate == 2);
    print("ok\n");
}
