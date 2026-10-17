#import "Basic";
#import "Sound_Player";

released: int;

release :: (stream: *Sound_Stream, data: *Sound_Data) {
    assert(find(stream.entity_id, stream.manager_id) == null);
    released += 1;
}

new_stream :: (data: *Sound_Data, id: s64, manager: s64, protected: bool = false) -> *Sound_Stream {
    result := make_stream(data);

    assert(result != null);
    result.entity_id = id;
    result.manager_id = manager;
    if protected result.user_flags |= .DO_NOT_KILL;

    return result;
}

main :: () {
    config: Sound_Player_Config;
    config.update_from_a_thread = false;
    config.release_asset = release;
    set_master_volume(0);

    if !sound_player_init(config) {
        sound_player_shutdown();
        print("native-device-unavailable\nok\n");
        return;
    }

    print("native-device-opened\n");
    samples: [1] s16;
    samples[0] = 16000;
    data: Sound_Data;
    data.loaded = true;
    data.type = .LINEAR_SAMPLE_ARRAY;
    data.samples = samples.data;
    data.sampling_rate = cast(u32) get_audio_sampling_rate();
    data.nsamples_times_nchannels = 1;
    done := new_stream(*data, 11, 1);
    survivor := new_stream(*data, 12, 1, true);

    // Nonzero samples are pulled only into our buffer. By the first post call
    // both cursors are beyond source bounds, so native submissions stay silent.
    set_master_volume(1);

    start_playing(done);
    start_playing(survivor);
    output: [32] u8;

    fill_sample_buffer(.{ output.count, output.data });
    assert(find(11, 1) == done && released == 0, "completion waits for entity retirement");
    pre_entity_update();
    assert((done.internal_flags & .FADING_OUT) != 0);
    post_entity_update(0.01);
    assert(released == 0 && find(11, 1) == done, "fading waits for a silent mixer frame");
    fill_sample_buffer(.{ output.count, output.data });
    post_entity_update(0.01);
    assert(released == 1 && find(11, 1) == null && find(12, 1) == survivor);
    stop_stream_abruptly(12);
    assert(released == 2, "explicit stop overrides DO_NOT_KILL");
    set_master_volume(0);

    other := data;
    a := new_stream(*data, 20, 2);
    b := new_stream(*data, 20, 3);
    c := new_stream(*other, 21, 2);

    set_repeating(a, true);
    set_repeating(b, true);
    set_repeating(c, true);
    start_playing(a);
    start_playing(b);
    start_playing(c);
    stop_stream_abruptly(20);
    assert(released == 4 && find(20, 2) == null && find(20, 3) == null);
    a = new_stream(*data, 22, 4);
    b = new_stream(*other, 23, 4);
    set_repeating(a, true);
    set_repeating(b, true);
    start_playing(a);
    start_playing(b);
    stop_stream_abruptly(*data);
    assert(released == 5 && find(22, 4) == null && find(21, 2) == c && find(23, 4) == b);
    stop_all_sounds_abruptly(4);
    assert(released == 6 && find(23, 4) == null && find(21, 2) == c);
    sound_player_shutdown();
    assert(released == 7 && !decode_queue_initted);
    print("ok\n");
}
