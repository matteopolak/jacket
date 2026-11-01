SP :: #import "Sound_Player";

Chunk :: struct {
    samples: [] s16;
    data: SP.Sound_Data;
    stream: *SP.Sound_Stream;
    start: float64;
    serial: s64;
}

Playback :: struct {
    reader: Audio.Reader;
    mixer: Mixer;
    opened: bool;
    device: bool;
    paused: bool = true;
    volume: float = 0.6;
    chunks: [3] Chunk;
    serial: s64;
    position: float64;
    error: string;
}

init_playback :: (p: *Playback) -> bool {
    config: SP.Sound_Player_Config;
    // All stream reads/writes and rendering happen on the UI thread. The backend buffers
    // ~40ms; the three application chunks hold up to half a second, never a whole track.
    config.update_from_a_thread = false;
    config.release_asset = release_chunk;
    p.device = SP.sound_player_init(config);
    for *p.chunks it.samples = NewArray(8192 * 2, s16);

    return p.device;
}

release_chunk :: (stream: *SP.Sound_Stream, data: *SP.Sound_Data) {
    chunk := cast(*Chunk) stream.user_data_pointer;
    if chunk chunk.stream = null;
}

stop_chunks :: (_p: *Playback) {
    SP.stop_all_sounds_abruptly(731);
}

close_track :: (p: *Playback) {
    stop_chunks(p);

    if p.opened {
        free_mixer(*p.mixer);
        Audio.close(*p.reader);
    }

    p.opened = false;
    p.paused = true;
    p.position = 0;
}

close_playback :: (p: *Playback) {
    close_track(p);
    SP.sound_player_shutdown();
    for *p.chunks array_free(it.samples);
}

open_track :: (p: *Playback, path: string) -> bool {
    close_track(p);
    p.error = "";
    if !p.device {
        p.error = "No audio output device; library and waveforms remain available.";
        return false;
    }

    if !Audio.open(*p.reader, path) {
        p.error = "Cannot decode this recording.";
        return false;
    }

    p.opened = true;
    init_mixer(*p.mixer, *p.reader, cast(s32) SP.get_audio_sampling_rate());
    p.paused = false;
    pump(p);

    return true;
}

seek_playback :: (p: *Playback, seconds: float64) {
    if !p.opened return;

    stop_chunks(p);
    seek_mixer(*p.mixer, seconds);
    p.position = p.mixer.cursor / p.reader.rate;
    pump(p);
}

toggle_pause :: (p: *Playback) {
    p.paused = !p.paused;
    for *p.chunks {
        if !it.stream continue;
        if p.paused it.stream.user_flags |= .PAUSED_DUE_TO_MENU;
        else it.stream.user_flags &= ~ .PAUSED_DUE_TO_MENU;
    }
}

// Returns true once every scheduled frame has played, including the final partial chunk.
pump :: (p: *Playback) -> bool {
    if !p.opened return false;

    first: *Chunk;
    last: *Chunk;

    for *p.chunks {
        if !it.stream continue;

        cursor := it.stream.play_cursor;
        if cursor >= it.data.nsamples_times_nchannels / 2 {
            p.position = max(p.position, it.start + cast(float64) (it.data.nsamples_times_nchannels / 2) / p.mixer.output_rate);
            SP.stop_stream_abruptly(*it.data);
            continue;
        }

        if !first || it.serial < first.serial first = it;
        if !last || it.serial > last.serial last = it;
    }

    if first && first.stream.silent_samples_at_start_of_play == 0 {
        p.position = first.start + first.stream.play_cursor / p.mixer.output_rate;
    }

    for *chunk: p.chunks {
        if chunk.stream continue;

        start := p.mixer.cursor / p.reader.rate;
        frames := mix(*p.mixer, chunk.samples);

        if p.mixer.failed {
            p.error = "Read or decode failed; playback stopped.";
            stop_chunks(p);
            p.paused = true;
            return false;
        }

        if frames == 0 continue;

        chunk.start = start;
        chunk.serial = p.serial;
        p.serial += 1;
        chunk.data = .{ loaded = true, type = .LINEAR_SAMPLE_ARRAY,
            nchannels = 2, sampling_rate = cast(u32) p.mixer.output_rate,
            samples = chunk.samples.data, nsamples_times_nchannels = frames * 2 };
        chunk.data.source_directions[0] = .FRONT_LEFT;
        chunk.data.source_directions[1] = .FRONT_RIGHT;
        chunk.stream = SP.make_stream(*chunk.data, .MUSIC);
        if !chunk.stream {
            p.error = "Could not create audio stream.";
            return false;
        }

        chunk.stream.manager_id = 731;
        chunk.stream.user_data_pointer = chunk;

        if last {
            remaining := cast(float64) (last.data.nsamples_times_nchannels / 2) - last.stream.play_cursor;
            chunk.stream.silent_samples_at_start_of_play = last.stream.silent_samples_at_start_of_play + cast(s64) max(cast(float64) 0, remaining);
        }

        if p.paused chunk.stream.user_flags |= .PAUSED_DUE_TO_MENU;

        chunk.stream.user_volume_scale = p.volume;
        SP.start_playing(chunk.stream);
        last = chunk;
    }

    for *p.chunks if it.stream it.stream.user_volume_scale = p.volume;

    SP.update();
    for p.chunks if it.stream return false;

    p.position = cast(float64) p.reader.frames / p.reader.rate;

    return true;
}
