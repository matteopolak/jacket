#import "Basic"(ENABLE_ASSERT = false);
#import "Sound_Player";
#import "Thread";

test_lib :: #library "jai_audio_output";
fail_close :: () #foreign test_lib "audio_test_fail_next_close";
fail_play :: () #foreign test_lib "audio_test_fail_next_play";
released: int;
errors: int;

probe :: (thread: *Thread) -> s64 {
    check(get_audio_sampling_rate() == 0);
    return 0;
}

check :: (condition: bool, message := "failure-preservation check failed") {
    if !condition {
        print("%\n", message, to_standard_error = true);
        exit(1);
    }
}

log_message :: (message: string, data: *void, info: Log_Info) {
    if (info.common_flags & .ERROR) != 0 {
        errors += 1;
        // A different thread proves callbacks do not hold recursive locks.
        worker: Thread;
        created := thread_init(*worker, probe);

        check(created);
        thread_start(*worker);
        joined := thread_is_done(*worker, 1000);

        check(joined);
        thread_deinit(*worker);
    }
}

release :: (stream: *Sound_Stream, data: *Sound_Data) {
    released += 1;
}

main :: () {
    context.logger = log_message;
    config: Sound_Player_Config;
    config.update_from_a_thread = true;
    config.release_asset = release;
    check(sound_player_init(config));
    pcm: [256] s16;
    data: Sound_Data;
    data.loaded = true;
    data.type = .LINEAR_SAMPLE_ARRAY;
    data.samples = pcm.data;
    data.nsamples_times_nchannels = pcm.count;
    stream := make_stream(*data);

    check(stream != null);
    stream.user_flags |= .DO_NOT_KILL | .REPEATING;
    start_playing(stream);
    fail_close();
    sound_player_shutdown();
    check(errors == 1 && released == 0);
    check(get_audio_sampling_rate() == 0);
    // A failed retained handle cannot be mistaken for a fresh successful init.
    check(!sound_player_init(config));
    sound_player_shutdown();
    check(released == 1 && !decode_queue_initted);
    check(sound_player_init(config));
    sound_player_shutdown();

    config.update_from_a_thread = false;
    check(sound_player_init(config));
    fail_play();
    fail_close();
    backend_play();
    check(errors == 2 && get_audio_sampling_rate() == 0);
    properties := backend_init(config);

    check(!properties.initted, "known-lost retained output must not report initialized");
    backend_shutdown();
    sound_player_shutdown();
    print("ok\n");
}
