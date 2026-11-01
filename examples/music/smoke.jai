// The real binary's non-interactive path. Generated PCM has an independent checksum in
// tools/smoke.sh; UI actions use the very same build procedure as the window.
run_smoke :: (folder: string) {
    path := tprint("%/sweep.wav", folder);
    reader: Audio.Reader;

    assert(Audio.open(*reader, path));
    defer Audio.close(*reader);
    mixer: Mixer;

    init_mixer(*mixer, *reader, 16000);
    defer free_mixer(*mixer);
    output := NewArray(32000, s16);

    defer array_free(output);
    frames := mix(*mixer, output, 0.5);

    assert(!mixer.failed && frames == 16000);
    print("offline frames=% checksum=%\n", frames, checksum(output));
    seek_mixer(*mixer, 0.5);
    short: [8] s16;

    assert(mix(*mixer, short, 0.5) == 4);
    for short assert(it == output[16000 + it_index]);

    app := New(App);
    app.offline = true;
    app.library = scan_library(folder);

    defer {
        close_app(app);
        free(app);
    }

    assert(app.library.tracks.count > 0);
    enqueue(*app.queue, 0);
    app.queue.cursor = 0;
    select_track(app, 0);
    while !poll_waveform(*app.wave) sleep_milliseconds(1);

    app.wave_ready = true;
    assert(app.wave.ok && app.wave.peaks[64] > 0);
    ui := headless(.{ width = 1100, height = 760 });

    defer free_ui(*ui);
    frame(*ui, app, build);
    waveform := find_label(*ui, "Waveform");

    assert(waveform != null && waveform.rect.w > 0 && waveform.rect.h > 0);
    assert(find_label(*ui, app.lyrics[0].text).value == "selected");
    app.playback.position = 0.6;
    frame(*ui, app, build);
    assert(find_label(*ui, app.lyrics[1].text).value == "selected");
    assert(find_label(*ui, app.lyrics[0].text).value != "selected");
    click(*ui, find_label(*ui, "Volume -"));
    frame(*ui, app, build);
    assert(app.playback.volume < 0.6);
    click(*ui, find_label(*ui, "+"));
    frame(*ui, app, build);
    assert(app.queue.items.count == 2);
    click(*ui, find_label(*ui, "Shuffle"));
    frame(*ui, app, build);
    assert(app.queue.shuffle);
    click(*ui, find_label(*ui, "Repeat: OFF"));
    frame(*ui, app, build);
    assert(app.queue.repeat == .ALL);
    assert(app.lyrics.count == 2 && lyric_at(app.lyrics, 600) == 1);
    print("headless-ui waveform lyrics queue: ok\n");
}

run_device_smoke :: (folder: string) {
    playback := New(Playback);

    defer {
        close_playback(playback);
        free(playback);
    }

    if !init_playback(playback) {
        print("device: skipped (no audio device)\n");
        return;
    }

    playback.volume = 0;
    assert(open_track(playback, tprint("%/sweep.wav", folder)));

    for 0..100 {
        pump(playback);
        sleep_milliseconds(2);
    }

    assert(playback.position > 0);
    toggle_pause(playback);
    seek_playback(playback, 0.5);
    assert(playback.paused && playback.position >= 0.5);
    toggle_pause(playback);

    for 0..50 {
        pump(playback);
        sleep_milliseconds(2);
    }

    assert(playback.error.count == 0);
    print("device: muted playback pause seek resume: ok\n");
}
