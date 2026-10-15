#import "Basic";
#import "File";
#import "Thread";
#import "Sound_Player";

data: Sound_Data;
completed: [2] s64;
released: int;

release :: (stream: *Sound_Stream, source: *Sound_Data) {
    assert(source == *data);
    released += 1;
}

build_streams :: (thread: *Thread) -> s64 {
    index := cast(s64) thread.data;

    for 0..15 {
        stream := make_stream(*data);

        assert(stream && stream.num_channels == 2 && stream.repeat_end_position == 4416);
        assert(stream.decoder && stream.decoder.num_channels == 2);
        assert(stream.current_rate > 0);
        deinit(stream.decoder);
        free(stream);
        completed[index] += 1;
    }

    return 0;
}

main :: () {
    bytes, read_ok := read_entire_file("fixtures/tone.ogg");

    assert(read_ok);
    defer free(bytes);
    data = load_audio_data("concurrent own fixture", bytes);
    assert(data.loaded);
    config: Sound_Player_Config;
    config.release_asset = release;
    set_master_volume(0);

    if !sound_player_init(config) {
        sound_player_shutdown();
        print("native-device-unavailable\nok\n");
        return;
    }

    print("native-device-opened\n");
    stream := make_stream(*data);

    assert(stream != null);
    stream.user_flags |= .DO_NOT_KILL | .REPEATING;
    start_playing(stream);
    workers: [2] Thread;
    for *worker: workers {
        created := thread_init(worker, build_streams);
        assert(created);
        worker.data = cast(*void) it_index;
        thread_start(worker);
    }

    for *worker: workers {
        joined := thread_is_done(worker, -1);
        assert(joined);
        thread_deinit(worker);
    }

    assert(completed[0] == 16 && completed[1] == 16);
    sound_player_shutdown();
    assert(released == 1 && data.nchannels == 2 && data.sampling_rate == 44100);
    print("ok\n");
}
