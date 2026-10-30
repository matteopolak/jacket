// test: only(.LINUX,.MACOS)
#import "Basic";
#import "Sound_Player";

released: int;
released_for: *Sound_Data;

release :: (stream: *Sound_Stream, data: *Sound_Data) {
    released += 1;
    released_for = data;
}

make_data :: (samples: *s16, count: s64, bytes: string) -> Sound_Data {
    data: Sound_Data;
    data.loaded = true;
    data.type = .LINEAR_SAMPLE_ARRAY;
    data.samples = samples;
    data.nsamples_times_nchannels = count;
    data.buffer = bytes;

    return data;
}

main :: () {
    config: Sound_Player_Config;
    config.update_from_a_thread = false;
    config.release_asset = release;

    if !sound_player_init(config) {
        sound_player_shutdown();
        print("native-device-unavailable\nok\n");
        return;
    }

    print("native-device-opened\n");
    set_master_volume(0);

    old_samples: [2048] s16;
    new_samples: [4096] s16;
    data := make_data(old_samples.data, old_samples.count, copy_string("old"));
    pointer := *data;

    // A sound that is playing is stopped, and the pointer to it stays good for the new one.
    stream := make_stream(pointer);

    assert(stream != null);
    stream.entity_id = 7;
    set_repeating(stream, true);
    start_playing(stream);
    assert(find(7, 0) == stream);

    assert(replace_sound_data(pointer, make_data(new_samples.data, new_samples.count, copy_string("new"))));
    assert(released == 1 && released_for == pointer);
    assert(find(7, 0) == null);
    assert(pointer == *data && data.samples == new_samples.data && data.buffer == "new");

    again := make_stream(pointer);

    assert(again != null && again.sound_data == pointer);
    again.entity_id = 8;
    set_repeating(again, true);
    start_playing(again);

    // A replacement that did not load changes nothing and stops nothing.
    broken: Sound_Data;

    assert(!replace_sound_data(pointer, broken));
    assert(released == 1 && find(8, 0) == again && data.buffer == "new");

    release_sound_data(pointer);
    assert(released == 2 && find(8, 0) == null);
    assert(!data.loaded && data.buffer.count == 0 && data.samples == null);
    assert(make_stream(pointer) == null);

    sound_player_shutdown();
    print("ok\n");
}
