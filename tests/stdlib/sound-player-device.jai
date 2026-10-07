// Sound_Player with the platform's audio output (Core Audio on macOS, DirectSound on Windows,
// ALSA's default PCM on Linux). Machines without an audio device fail to open it: init must then
// report false and leave the player stopped, so every other call is a quiet no-op. With a device,
// a silent stream plays for a moment (master volume 0) and the player shuts down cleanly. The
// Linux CI runners have no sound card and play into ALSA's null PCM. wasm has no audio (see the skip
// list).
#import "Basic";

#import "Sound_Player";

released: int;
note_release :: (stream: *Sound_Stream, data: *Sound_Data) {
    released += 1;
}

main :: () {
    devices := get_devices();
    for devices assert(it.name.count > 0, "device % has no name", it_index);

    // A short mono WAV of silence.
    wav: String_Builder;
    header :: u8.[
        #char "R", #char "I", #char "F", #char "F", 36 + 200, 0, 0, 0,
        #char "W", #char "A", #char "V", #char "E", #char "f", #char "m", #char "t", #char " ",
        16, 0, 0, 0, 1, 0, 1, 0, 0x44, 0xac, 0, 0, 0x88, 0x58, 1, 0, 2, 0, 16, 0,
        #char "d", #char "a", #char "t", #char "a", 200, 0, 0, 0];
    for header append(*wav, it);
    for 1..200 append(*wav, 0);
    data := load_audio_data("silence.wav", builder_to_string(*wav));
    assert(data.loaded && data.sampling_rate == 44100 && data.nsamples_times_nchannels == 100);

    config: Sound_Player_Config;
    config.release_asset = note_release;
    if !sound_player_init(config) {
        // No device: nothing runs, and nothing crashes.
        assert(get_audio_sampling_rate() == 0, "a failed init left a sampling rate");
        stream := make_stream(*data);
        start_playing(stream); // Not running: ignored.
        update();
        assert(find(0, 0) == null && released == 0);
        free(stream);
        sound_player_shutdown();
        print("ok\n");
        return;
    }
    assert(get_audio_sampling_rate() > 0, "running with sampling rate %", get_audio_sampling_rate());
    set_master_volume(0);
    stream := make_stream(*data);
    set_repeating(stream, true);
    stream.entity_id = 5;
    start_playing(stream);
    for 1..10 {
        update();
        sleep_milliseconds(5);
    }
    assert(find(5, 0) == stream, "the stream stopped by itself");
    sound_player_shutdown();
    assert(released == 1, "shutdown released % streams", released);
    assert(get_devices().count == devices.count);
    print("ok\n");
}
