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

    for channel: 0..data.nchannels - 1 data.source_directions[channel] = directions[channel];
}

report_audio_error :: (name: string, detail: string) {
    context.logger(tprint("audio %: %\n", name, detail), context.logger_data, Log_Info.{ common_flags = .ERROR });
}
