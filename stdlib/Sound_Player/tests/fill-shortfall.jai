// test: only(.WINDOWS,.LINUX,.MACOS)
// One producer turn refills the whole shortfall, not one chunk of it, so a late frame does not
// leave the output short until the next turn.
#import "Basic";
#import "Sound_Player";

main :: () {
    config: Sound_Player_Config;
    config.update_from_a_thread = false;
    config.audible_when_window_is_not_in_focus = true;
    config.seconds_to_fill_ahead = 0.2; // several chunks' worth
    set_master_volume(0);

    if !sound_player_init(config) {
        sound_player_shutdown();
        print("native-device-unavailable\nok\n");
        return;
    }

    defer sound_player_shutdown();
    print("native-device-opened\n");

    // Let the silence queued at start play out, so the shortfall is the whole target.
    buffered, minimum := backend_count_buffered_bytes();

    for 0..99 {
        if buffered == 0 break;
        sleep_milliseconds(20);
        buffered, minimum = backend_count_buffered_bytes();
    }

    assert(buffered == 0, "the start-up silence did not play out");

    silence := s16.[0];
    data := Sound_Data.{ loaded = true, type = .LINEAR_SAMPLE_ARRAY, samples = silence.data, nsamples_times_nchannels = silence.count };
    stream := make_stream(*data);

    assert(stream != null);
    set_repeating(stream, true);
    start_playing(stream);

    update_from_async_thread(false);

    frame_bytes := backend_init(config).num_channels * 2;

    assert(stream.samples_streamed_since_entity_update == minimum / frame_bytes, "one turn mixed % of % frames", stream.samples_streamed_since_entity_update, minimum / frame_bytes);
    print("ok\n");
}
