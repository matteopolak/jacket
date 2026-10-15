#import "Basic";
#import "Sound_Player";

released: int;
last_streamed: s64;
priority_calls: int;

set_priority :: () {
    priority_calls += 1;
}

release :: (stream: *Sound_Stream, data: *Sound_Data) {
    assert(stream.sound_data == data);
    assert(find(stream.entity_id, stream.manager_id) == null);
    last_streamed = stream.samples_streamed_since_entity_update;
    released += 1;
}

main :: () {
    assert(get_audio_sampling_rate() == 0);
    config: Sound_Player_Config;
    config.keep_sounds_alive_by_default = false;
    config.update_from_a_thread = false;
    config.release_asset = release;
    config.set_async_thread_priority = set_priority;
    if !sound_player_init(config) {
        sound_player_shutdown();
        print("native-device-unavailable\nok\n");
        return;
    }

    print("native-device-opened\n");
    assert(get_audio_sampling_rate() > 0);
    set_master_volume(0);
    samples: [2048] s16;
    data: Sound_Data;
    data.loaded = true;
    data.type = .LINEAR_SAMPLE_ARRAY;
    data.samples = samples.data;
    data.nsamples_times_nchannels = samples.count;
    a := make_stream(*data);
    b := make_stream(*data, .MUSIC);

    assert(a && b && a.category == .GENERAL_SFX && b.category == .MUSIC);
    a.entity_id = 1;
    a.manager_id = 10;
    b.entity_id = 2;
    b.manager_id = 10;
    set_repeating(a, true);
    set_repeating(b, true);
    start_playing(a);
    start_playing(a);
    start_playing(b);
    assert(find(1, 10) == a && find(2, 10) == b);
    pre_entity_update();
    b.marked = true;
    post_entity_update(0.01);
    assert(find(1, 10) == null && find(2, 10) == b && released == 1);
    stop_all_sounds_abruptly(10);
    assert(released == 2 && find(2, 10) == null);
    // Objects started from a temporary caller context remain durable.
    push_allocator(temp);

    for i: 0..15 {
        stream := make_stream(*data);
        stream.entity_id = i + 100;
        set_repeating(stream, true);
        start_playing(stream);
        reset_temporary_storage();
        assert(find(i + 100, 0) == stream);
        stop_stream_abruptly(i + 100);
    }

    sound_player_shutdown();
    assert(released == 18 && get_audio_sampling_rate() == 0 && !decode_queue_initted);
    config.update_from_a_thread = true;
    config.keep_sounds_alive_by_default = true;
    assert(sound_player_init(config));
    stream := make_stream(*data);

    set_repeating(stream, true);
    start_playing(stream);
    sleep_milliseconds(100);
    sound_player_shutdown();
    assert(released == 19 && last_streamed > 0 && priority_calls == 1);
    print("ok\n");
}
