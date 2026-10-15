#import "Basic";
#import "Thread";
#import "Sound_Player";

released: int;

probe :: (thread: *Thread) -> s64 {
    set_master_volume(0);
    update_from_async_thread(false);
    return 0;
}

check_unlocked :: () {
    worker: Thread;

    assert(thread_init(*worker, probe));
    thread_start(*worker);
    assert(thread_is_done(*worker, 1000), "phase/callback must not retain stream or producer locks");
    thread_deinit(*worker);
}

release :: (stream: *Sound_Stream, data: *Sound_Data) {
    assert(find(77, 0) == null);
    assert(stream.samples_streamed_since_entity_update > 0, "mixer must progress after pre returns");
    check_unlocked();
    stop_stream_abruptly(77);
    released += 1;
}

main :: () {
    config: Sound_Player_Config;
    config.release_asset = release;
    set_master_volume(0);

    if !sound_player_init(config) {
        sound_player_shutdown();
        print("native-device-unavailable\nok\n");
        return;
    }

    print("native-device-opened\n");
    samples: [32768] s16;
    data: Sound_Data;
    data.loaded = true;
    data.type = .LINEAR_SAMPLE_ARRAY;
    data.samples = samples.data;
    data.sampling_rate = cast(u32) get_audio_sampling_rate();
    data.nsamples_times_nchannels = samples.count;
    stream := make_stream(*data);
    stream.entity_id = 77;
    set_repeating(stream, true);
    start_playing(stream);

    // Each call is a complete synchronized phase, including calls made alone.
    post_entity_update(0.01);
    check_unlocked();
    pre_entity_update();
    check_unlocked();
    pre_entity_update();
    check_unlocked();
    set_repeating(stream, false);
    set_repeating(stream, true);
    sleep_milliseconds(200);
    stop_stream_abruptly(77);
    assert(released == 1 && find(77, 0) == null);
    post_entity_update(0.01);
    assert(released == 1);
    pre_entity_update();
    sound_player_shutdown();
    print("ok\n");
}
