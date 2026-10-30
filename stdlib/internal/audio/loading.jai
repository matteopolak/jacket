load_audio_file :: (filename: string, optional := false) -> Sound_Data {
    bytes, success := F.read_entire_file(filename);
    if !success {
        if !optional report_audio_error(filename, "could not read file");
        return .{};
    }

    result := load_audio_data(filename, bytes);
    result.full_path = filename;

    return result;
}

load_audio_data :: (name_for_error_reporting: string, data: string) -> Sound_Data {
    result: Sound_Data;
    result.name = name_for_error_reporting;
    result.buffer = data;

    if data.count < 4 {
        report_audio_error(name_for_error_reporting, "truncated audio signature");
        return result;
    }

    if get_magic4(data, 0) == 0x5367674f {
        result.type = .OGG_COMPRESSED;
        result.loaded = true;
        return result;
    }

    format, samples, success, extra := Wav.get_wav_header(data);
    if !success || format.nChannels <= 0 || format.nChannels > MAX_CHANNELS_PER_SOUND || format.nSamplesPerSec <= 0 {
        report_audio_error(name_for_error_reporting, "unsupported or malformed audio header");
        return result;
    }

    result.nchannels = cast(u16) format.nChannels;
    result.sampling_rate = cast(u32) format.nSamplesPerSec;

    if format.wFormatTag == Wav.WAVE_FORMAT_PCM {
        if format.wBitsPerSample != 16 || format.nBlockAlign != format.nChannels * 2 || samples.count % format.nBlockAlign != 0 {
            report_audio_error(name_for_error_reporting, "invalid 16-bit PCM format");
            return result;
        }

        result.type = .LINEAR_SAMPLE_ARRAY;
        result.nsamples_times_nchannels = samples.count / 2;
        result.samples = cast(*s16) samples.data;
    } else if format.wFormatTag == Wav.WAVE_FORMAT_DVI_ADPCM {
        result.type = .ADPCM_COMPRESSED;
        result.nBlockAlign = format.nBlockAlign;
        result.wSamplesPerBlock = extra.wSamplesPerBlock;
        if result.nBlockAlign <= 0 || result.wSamplesPerBlock <= 0 {
            report_audio_error(name_for_error_reporting, "invalid ADPCM block metadata");
            return result;
        }

        frames := samples.count / result.nBlockAlign * result.wSamplesPerBlock;
        if extra.wSamplesAccordingToFactChunk > 0 frames = min(frames, cast(s64) extra.wSamplesAccordingToFactChunk);

        result.nsamples_times_nchannels = frames * result.nchannels;
        if !valid_adpcm(*result) {
            report_audio_error(name_for_error_reporting, "inconsistent ADPCM data");
            return result;
        }
    } else {
        report_audio_error(name_for_error_reporting, "unsupported WAV codec");
        return result;
    }

    assign_channel_directions(*result);
    result.loaded = true;

    return result;
}

// Stops every stream that plays `data`, frees the bytes it was loaded from and clears it, so that nothing points into it any more.
//
// The player reads the sound's `buffer` (and the `samples` and decoder state that point into it) for as long as a stream
// uses it, so a loaded sound cannot just be freed while it plays. This stops those streams first, the way
// `stop_stream_abruptly(data)` does: `Sound_Player_Config.release_asset` is called for each, and the `*Sound_Stream` pointers
// the program holds for them are dead after that, so drop them there. Then `buffer` is freed with `context.allocator`, which
// is where `load_audio_file` got it, and `data` is reset to an empty `Sound_Data`. Call it from the thread that updates the player.
//
// Do not use it on data whose `buffer` the program owns some other way (a constant, an arena): stop the streams with
// `stop_stream_abruptly(data)` and leave the memory alone. A stream made with `make_stream` but not yet started is not known to
// the player and must be started or dropped by the caller first.
//
// ```jai,skip_on(wasm)
// samples: [64] s16;
// data := Sound_Data.{ loaded = true, type = .LINEAR_SAMPLE_ARRAY, samples = samples.data, nsamples_times_nchannels = samples.count };
// data.buffer = copy_string("stands in for the file bytes");
//
// release_sound_data(*data);
// assert(!data.loaded && data.buffer.count == 0);
// ```
release_sound_data :: (data: *Sound_Data) {
    if !data return;

    stop_stream_abruptly(data);
    free(data.buffer.data);
    data.* = .{};
}

// Swaps in a new sound for `data` without moving it: the streams playing the old one are stopped, its bytes are freed and
// `replacement` takes its place. Anything that holds the `*Sound_Data` keeps a valid pointer and gets the new sound from the next
// `make_stream` on. This is how to hot-reload a sound.
//
// The old sound goes as in `release_sound_data`: its streams end (a stream that was playing does not carry on with the new
// data; start a new one), `Sound_Player_Config.release_asset` is called for each, and the old `buffer` is freed with
// `context.allocator`. `replacement` is typically `load_audio_file(path)` or `load_audio_data(name, bytes)`; one that did not
// load (`loaded` is `false`) is refused with `false` and `data` is left playing as it was, so a bad file on disk does not silence
// a good sound. A `replacement` that was refused still owns its bytes: free them.
//
// ```jai,skip_on(wasm)
// first := load_audio_data("first", copy_string("not audio"));
// assert(!first.loaded); // too short to be a sound; its bytes are still ours
// free(first.buffer.data);
//
// samples: [64] s16;
// data := Sound_Data.{ loaded = true, type = .LINEAR_SAMPLE_ARRAY, samples = samples.data, nsamples_times_nchannels = samples.count };
// data.buffer = copy_string("old bytes");
//
// louder: [64] s16;
// louder[0] = 100;
// replacement := Sound_Data.{ loaded = true, type = .LINEAR_SAMPLE_ARRAY, samples = louder.data, nsamples_times_nchannels = louder.count };
// replacement.buffer = copy_string("new bytes");
//
// assert(replace_sound_data(*data, replacement));
// assert(data.samples == louder.data && data.buffer == "new bytes");
// release_sound_data(*data);
// ```
replace_sound_data :: (data: *Sound_Data, replacement: Sound_Data) -> bool {
    if !data || !replacement.loaded return false;

    stop_stream_abruptly(data);
    free(data.buffer.data);
    data.* = replacement;

    return true;
}

#scope_module
assign_channel_directions :: (data: *Sound_Data) {
    // The API defines no dedicated seven-channel/center-rear layout.
    // Preserve configured directions; callers can set them before stream start.
    if data.nchannels == 7 return;

    if data.nchannels == 1 {
        data.source_directions[0] = .MONO;
        return;
    }

    directions := Source_Direction.[.FRONT_LEFT, .FRONT_RIGHT, .CENTER_FORWARD, .SUBWOOFER_CENTER_FORWARD, .REAR_LEFT, .REAR_RIGHT, .EXTEND_LEFT, .EXTEND_RIGHT];

    if data.type == .OGG_COMPRESSED {
        if data.nchannels == 3 || data.nchannels >= 5 {
            directions[1] = .CENTER_FORWARD;
            directions[2] = .FRONT_RIGHT;
            directions[3] = .REAR_LEFT;
            directions[4] = .REAR_RIGHT;
            directions[5] = .SUBWOOFER_CENTER_FORWARD;
        } else if data.nchannels == 4 {
            directions[2] = .REAR_LEFT;
            directions[3] = .REAR_RIGHT;
        }

        if data.nchannels == 8 {
            directions[3] = .EXTEND_LEFT;
            directions[4] = .EXTEND_RIGHT;
            directions[5] = .REAR_LEFT;
            directions[6] = .REAR_RIGHT;
            directions[7] = .SUBWOOFER_CENTER_FORWARD;
        }
    }

    // Both loaders reject more than MAX_CHANNELS_PER_SOUND channels, the size of both arrays.
    assert(data.nchannels <= directions.count);
    memcpy(data.source_directions.data, directions.data, data.nchannels * size_of(Source_Direction));
}

report_audio_error :: (name: string, detail: string) {
    context.logger(tprint("audio %: %\n", name, detail), context.logger_data, Log_Info.{ common_flags = .ERROR });
}
