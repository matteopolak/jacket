// Pure Sound_Player contracts; this case does not initialize a native audio device.
#import "Basic";
Sound_Player :: #import "Sound_Player";

main :: () {
    data: Sound_Player.Sound_Data;

    assert(data.nchannels == 1 && data.sampling_rate == 44100);
    assert(data.volume_scale == 1 && data.type == .UNINITIALIZED);

    stream: Sound_Player.Sound_Stream;

    assert(stream.num_channels == 2 && stream.repeat_end_position == -1);
    assert(stream.inner_radius == 0.5 && stream.outer_radius == 10);
    assert(stream.current_rate == 1 && stream.desired_rate == 1);

    assert(!Sound_Player.skip_this_output_channel(.REAR_LEFT, *stream));
    stream.user_flags |= .IS_FOOTSTEP;
    assert(Sound_Player.skip_this_output_channel(.REAR_LEFT, *stream));
    assert(Sound_Player.skip_this_output_channel(.REAR_RIGHT, *stream));
    assert(!Sound_Player.skip_this_output_channel(.FRONT_LEFT, *stream));
    assert(!Sound_Player.skip_this_output_channel(.SUBWOOFER, *stream));

    data.nsamples_times_nchannels = 4;
    bounds: Sound_Player.Buffer_Bounds;
    success, no_overflow := Sound_Player.get_buffer_bounds(*stream, *data, 1, *bounds);

    assert(success && no_overflow);
    assert(bounds.start == 0 && bounds.end == 4 && bounds.memory == null);

    stream.repeat_end_position = 3;
    success, no_overflow = Sound_Player.get_buffer_bounds(*stream, *data, 1, *bounds);
    assert(success && no_overflow && bounds.end == 3);

    accumulator: [2] Sound_Player.Audio_Dest_Sample;
    accumulator[0].channels[0] = 17;
    result := Sound_Player.accumulate_stream(*stream, accumulator, 2);

    assert(!result && accumulator[0].channels[0] == 17);

    print("ok\n");
}
