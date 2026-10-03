TEST_K :: 1.0;
DELTA_RATE_MAX :: 18.0 * (1.0 / 40000.0) * TEST_K;
DELTA_RATE_MIN :: -DELTA_RATE_MAX;

samples_to_bytes :: (stream: *Sound_Stream, num_samples: s64) -> s64 {
    assert(stream && stream.num_channels > 0 && stream.num_channels <= MAX_CHANNELS_PER_SOUND);
    assert(num_samples >= 0 && num_samples <= 0x7fffffffffffffff / (stream.num_channels * 2));
    return num_samples * stream.num_channels * 2;
}

get_buffer_bounds :: (stream: *Sound_Stream, data: *Sound_Data, index: s64, result: *Buffer_Bounds) -> success: bool, no_overflow: bool {
    result.* = .{};
    if !stream || !data || data.nchannels == 0 return false, false;

    length := data.nsamples_times_nchannels / data.nchannels;
    if stream.repeat_end_position >= 0 length = min(length, stream.repeat_end_position);
    if index < 0 || index >= length return false, false;

    if data.type != .LINEAR_SAMPLE_ARRAY {
        if !stream.decoder return false, false;

        found, start, end, memory := find_page_containing_cursor(stream.decoder, index);
        if !found return false, false;

        result.start = start;
        result.end = min(end, length);
        result.memory = memory;

        return true, true;
    }

    if !data.samples return false, false;

    result.start = 0;
    result.end = length;
    result.memory = data.samples;

    return true, true;
}

sample_n_channels :: (stream: *Sound_Stream, samples: *s16, page_start: s64, page_end: s64, source_cursor: float64, samples_return: [] float, no_overflow: bool) #no_abc {
    assert(stream && samples && page_end > page_start);
    assert(samples_return.count >= stream.num_channels);
    index, remainder := get_icursor_and_remainder(source_cursor);

    assert(index >= page_start && index < page_end);
    next := min(index + 1, page_end - 1);

    for channel: 0..stream.num_channels - 1 {
        a := cast(float) read_pcm_sample(samples, (index - page_start) * stream.num_channels + channel) / 32768.0;
        b := cast(float) read_pcm_sample(samples, (next - page_start) * stream.num_channels + channel) / 32768.0;
        samples_return[channel] = a + (b - a) * cast(float) remainder;
    }
}

mix_sample_into_output_streams :: (stream: *Sound_Stream, source: *Audio_Source_Sample, dest: *Audio_Dest_Sample) #no_abc {
    assert(stream.num_channels > 0 && stream.num_channels <= MAX_CHANNELS_PER_SOUND);

    for input: 0..stream.num_channels - 1 {
        for output: 0..OUTPUT_CHANNELS_MAX - 1 {
            contribution := source.channels[input] * stream.input_scale_mappings[input].interpolated_source_scale_for_this_output_index[output];
            dest.channels[output] += contribution;
            if contribution != 0 stream.silent_this_frame = false;
        }
    }
}

maybe_wrap_play_cursor :: (stream: *Sound_Stream) {
    if !(stream.user_flags & .REPEATING) || !stream.sound_data return;

    data := stream.sound_data;
    if data.nchannels == 0 return;

    end := data.nsamples_times_nchannels / data.nchannels;
    if stream.repeat_end_position >= 0 end = min(end, stream.repeat_end_position);

    start := max(cast(s64) 0, stream.repeat_start_position);
    if end <= start || stream.play_cursor < end return;

    stream.play_cursor = start + fmod(stream.play_cursor - start, cast(float64) (end - start));
}

catch_up_volumes :: (stream: *Sound_Stream) {
    for *mapping: stream.input_scale_mappings {
        for output: 0..OUTPUT_CHANNELS_MAX - 1 {
            desired := mapping.source_scale_for_this_output_index[output];
            current := mapping.interpolated_source_scale_for_this_output_index[output];
            step := abs(mapping.dvolume_dsample[output]);

            if current < desired current = min(desired, current + step);
            else current = max(desired, current - step);

            mapping.interpolated_source_scale_for_this_output_index[output] = current;
        }
    }
}

accumulate_buffer_f :: (accumulator: *Audio_Dest_Sample, num_samples: s64, source_cursor: float64, page: *void, page_start: s64, page_end: s64, stream: *Sound_Stream, no_overflow: bool) -> (cursor: float64, samples_written: s64) {
    assert(num_samples >= 0);
    cursor := source_cursor;
    written: s64;

    while written < num_samples && cursor >= page_start && cursor < page_end {
        sample: Audio_Source_Sample;

        sample_n_channels(stream, cast(*s16) page, page_start, page_end, cursor, sample.channels, no_overflow);
        catch_up_volumes(stream);
        mix_sample_into_output_streams(stream, *sample, accumulator + written);
        stream.current_rate += clamp(stream.desired_rate - stream.current_rate, DELTA_RATE_MIN, DELTA_RATE_MAX);
        assert(is_finite(stream.current_rate));
        cursor += stream.current_rate;
        written += 1;
    }

    return cursor, written;
}

accumulate_stream :: (stream: *Sound_Stream, accumulator: [] Audio_Dest_Sample, num_samples: s64) -> bool {
    assert(num_samples >= 0 && num_samples <= accumulator.count);
    if !stream || !stream.sound_data return false;
    if stream.internal_flags & .WAITING_FOR_INITIAL_DECODER_PAGES return false;

    stream.silent_this_frame = true;
    if stream.user_flags & .PAUSED_DUE_TO_MENU {
        stream.user_flags |= .WAS_PAUSED_DUE_TO_MENU;
        return false;
    }

    if stream.user_flags & .WAS_PAUSED_DUE_TO_MENU {
        stream.current_rate = stream.desired_rate;
        stream.user_flags &= ~ .WAS_PAUSED_DUE_TO_MENU;
    }

    assert(is_finite(stream.current_rate) && is_finite(stream.desired_rate));
    data := stream.sound_data;
    if data.nchannels == 0 return false;

    end := data.nsamples_times_nchannels / data.nchannels;
    if stream.repeat_end_position >= 0 end = min(end, stream.repeat_end_position);
    if (stream.user_flags & .REPEATING) && end <= max(cast(s64) 0, stream.repeat_start_position) return false;

    written: s64;
    missing_page := false;

    while written < num_samples {
        if stream.silent_samples_at_start_of_play > 0 {
            count := min(num_samples - written, stream.silent_samples_at_start_of_play);

            written += count;
            stream.silent_samples_at_start_of_play -= count;
            stream.samples_streamed_since_entity_update += count;
            continue;
        }

        maybe_wrap_play_cursor(stream);

        if missing_page || stream.play_cursor < 0 || stream.play_cursor >= end {
            catch_up_volumes(stream);
            stream.current_rate += clamp(stream.desired_rate - stream.current_rate, DELTA_RATE_MIN, DELTA_RATE_MAX);
            stream.play_cursor += stream.current_rate;
            stream.samples_streamed_since_entity_update += 1;
            written += 1;
            continue;
        }

        bounds: Buffer_Bounds;
        found, safe := get_buffer_bounds(stream, data, cast(s64) stream.play_cursor, *bounds);

        if !found {
            // A missing page silences the remainder of this accumulation call.
            // Do not repeatedly contend on the decode queue for every frame.
            missing_page = true;
            continue;
        }

        cursor, count := accumulate_buffer_f(accumulator.data + written, num_samples - written, stream.play_cursor, bounds.memory, bounds.start, min(bounds.end, end), stream, safe);

        stream.play_cursor = cursor;
        written += count;
        stream.samples_streamed_since_entity_update += count;
        if count == 0 return false;
    }

    return false;
}

#scope_file
read_pcm_sample :: inline (samples: *s16, index: s64) -> s16 {
    bytes := (cast(*u8) samples) + index * 2;
    low := cast(u32) bytes[0];
    high := cast(u32) bytes[1];
    bits := low | (high << 8);

    return cast(s16) cast,truncate(u16) bits;
}

fmod :: (x: float64, y: float64) -> float64 {
    return x - floor(x / y) * y;
}

#import "Basic";
#import "Math";
