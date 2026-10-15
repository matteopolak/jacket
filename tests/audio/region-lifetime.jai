#import "Basic";
#import "Thread";
#import "Sound_Player";

produce :: (thread: *Thread) -> s64 {
    update_from_async_thread(false);
    return 0;
}

main :: () {
    config: Sound_Player_Config;
    config.update_from_a_thread = false;
    config.audible_when_window_is_not_in_focus = true;
    set_master_volume(0);

    if !sound_player_init(config) {
        sound_player_shutdown();
        print("native-device-unavailable\nok\n");
        return;
    }

    print("native-device-opened\n");
    buffered: s64;

    for 0..99 {
        buffered = backend_count_buffered_bytes();
        if buffered == 0 break;
        sleep_milliseconds(20);
    }

    assert(buffered == 0, "establish drained native silent buffers");
    properties := backend_init(config);
    regions, success := backend_lock_fill_regions(properties.num_channels * 2 * 256);

    assert(success && regions.buffer0.count > 0);
    memset(regions.buffer0.data, 0x5a, regions.buffer0.count);
    second, available := backend_lock_fill_regions(properties.num_channels * 2 * 256);

    assert(!available && second.buffer0.count == 0);
    worker: Thread;

    assert(thread_init(*worker, produce));
    thread_start(*worker);
    assert(thread_is_done(*worker, 1000), "producer must respect a borrowed region without retaining a caller lock");
    thread_deinit(*worker);
    for byte: regions.buffer0 assert(byte == 0x5a, "borrowed region must not be overwritten");

    buffered = backend_count_buffered_bytes();
    assert(buffered == 0, "buffer count must report actual queued bytes, not a reservation sentinel");
    memset(regions.buffer0.data, 0, regions.buffer0.count);
    backend_release_fill_regions(regions);
    sound_player_shutdown();
    print("ok\n");
}
