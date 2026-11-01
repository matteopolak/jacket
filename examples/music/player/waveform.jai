// Results are published only after thread_is_done joins the worker. Switching tracks joins
// before freeing the copied path; no UI pointers or playback decoder are shared with it.
Waveform :: struct {
    thread: Thread;
    running: bool;
    path: string;
    peaks: [128] float;
    ok: bool;
}

start_waveform :: (wave: *Waveform, path: string) -> bool {
    finish_waveform(wave);
    wave.* = .{ path = copy_string(path) };
    if !thread_init(*wave.thread, compute_waveform, name = "music waveform") {
        free(wave.path);
        wave.path = "";
        return false;
    }

    wave.running = true;
    wave.thread.data = wave;
    thread_start(*wave.thread);

    return true;
}

poll_waveform :: (wave: *Waveform) -> bool {
    if !wave.running return true;
    if !thread_is_done(*wave.thread, 0) return false;

    thread_deinit(*wave.thread);
    wave.running = false;

    return true;
}

finish_waveform :: (wave: *Waveform) {
    if wave.running {
        thread_is_done(*wave.thread, -1);
        thread_deinit(*wave.thread);
    }

    free(wave.path);
    wave.path = "";
    wave.running = false;
}

#scope_file
compute_waveform :: (thread: *Thread) -> s64 {
    wave := cast(*Waveform) thread.data;
    reader: Audio.Reader;

    if !Audio.open(*reader, wave.path) return 1;

    defer Audio.close(*reader);
    samples := NewArray(4096 * reader.channels, s16);

    defer array_free(samples);

    while reader.position < reader.frames {
        start := reader.position;
        got := Audio.read(*reader, samples);

        if got <= 0 return 1;

        for f: 0..got - 1 {
            bucket := min((start + f) * wave.peaks.count / reader.frames, wave.peaks.count - 1);

            for c: 0..reader.channels - 1 {
                peak := abs(cast(float) samples[f * reader.channels + c]) / 32768;
                wave.peaks[bucket] = max(wave.peaks[bucket], peak);
            }
        }
    }

    wave.ok = true;

    return 0;
}
