#import "Basic";
#import "Sound_Player";

errors: int;

capture_error :: (message: string, data: *void, info: Log_Info) {
    assert(message.count > 0 && (info.common_flags & .ERROR));
    errors += 1;
}

main :: () {
    wav := u8.[
        82, 73, 70, 70, 40, 0, 0, 0, 87, 65, 86, 69,
        102, 109, 116, 32, 16, 0, 0, 0, 1, 0, 1, 0,
        68, 172, 0, 0, 136, 88, 1, 0, 2, 0, 16, 0,
        100, 97, 116, 97, 4, 0, 0, 0, 0, 64, 0, 192
    ];
    data := load_audio_data("two samples", .{ wav.count, wav.data });

    assert(data.loaded && data.type == .LINEAR_SAMPLE_ARRAY);
    assert(data.nchannels == 1 && data.sampling_rate == 44100 && data.nsamples_times_nchannels == 2);
    assert(data.samples[0] == 16384 && data.samples[1] == -16384);
    wav[45] = 0;
    assert(data.samples[0] == 0, "loaded samples must alias supplied storage");
    assert(data.buffer.data == wav.data && data.buffer.count == wav.count);
    assert(get_magic4(.{ wav.count, wav.data }, 0) == 0x46464952);
    // Seven channels fit capacity; no invented center-rear layout is assigned.
    seven: [72] u8;

    memcpy(seven.data, wav.data, 44);
    seven[4] = 64;
    seven[22] = 7;
    seven[28] = 184;
    seven[29] = 107;
    seven[30] = 9;
    seven[32] = 14;
    seven[40] = 28;
    seven_data := load_audio_data("seven channels", .{ seven.count, seven.data });

    assert(seven_data.loaded && seven_data.nchannels == 7 && seven_data.nsamples_times_nchannels == 14);
    for direction: seven_data.source_directions assert(direction == .MONO);

    context.logger = capture_error;
    bad := load_audio_data("invalid", "not audio");

    assert(!bad.loaded && bad.type == .UNINITIALIZED);
    assert(errors == 1, "malformed audio must report an error");
    print("ok\n");
}
