get_devices :: () -> [] Output_Device {
    lock_producer();
    defer finish_backend_call();
    return backend_get_devices();
}

get_audio_sampling_rate :: () -> s64 {
    lock_producer();
    defer unlock_producer();
    lock_player();
    defer unlock_player();
    if !player_initted return 0;

    return backend_current_rate();
}

sound_player_init :: (given_config: Sound_Player_Config) -> bool {
    push_allocator(context.default_allocator);
    if player_initted return true;

    player_config = given_config;

    if !sound_mutex_ready {
        init(*sound_mutex);
        init(*producer_mutex);
        producer_mutex_ready = true;
        sound_mutex_ready = true;
    }

    init_sound_player_decode_queue();
    properties := backend_init(given_config);
    if !properties.initted {
        #if VERBOSE context.logger("Sound_Player: output unavailable\n", context.logger_data, Log_Info.{});
        return false;
    }

    player_initted = true;
    async_exit = false;
    last_entity_time = audio_wall_time();

    for 0..7 {
        buffered, minimum := backend_count_buffered_bytes();
        if buffered >= minimum break;
        update_from_async_thread(false);
    }

    if !start_output_if_possible() {
        #if VERBOSE context.logger("Sound_Player: output start failed\n", context.logger_data, Log_Info.{});
        sound_player_shutdown();
        return false;
    }

    if given_config.update_from_a_thread {
        if !thread_init(*audio_thread, async_audio_update, name = "audio output") {
            #if VERBOSE context.logger("Sound_Player: output worker unavailable\n", context.logger_data, Log_Info.{});
            sound_player_shutdown();
            return false;
        }

        audio_thread_started = true;
        thread_start(*audio_thread);
    }

    #if VERBOSE context.logger(tprint("Sound_Player: initialized % channels at % Hz\n", properties.num_channels, properties.output_sampling_rate), context.logger_data, Log_Info.{});

    return true;
}

sound_player_shutdown :: () {
    push_allocator(context.default_allocator);

    if sound_mutex_ready {
        lock(*sound_mutex);
        async_exit = true;
        unlock(*sound_mutex);

        if audio_thread_started {
            joined := thread_is_done(*audio_thread, -1);
            if !joined audio_fatal("audio output worker join failed");
            thread_deinit(*audio_thread);
            audio_thread_started = false;
        }

        lock_producer();
        lock(*sound_mutex);
        player_initted = false;
        while active_streams.count > 0 retire_stream_at(active_streams.count - 1);

        unlock(*sound_mutex);
        closed := close_output_device();
        if !closed {
            unlock_producer();
            report_backend_notice();
            return;
        }

        unlock_producer();
        release_retired_streams();
        array_free(active_streams);
        active_streams = .{};
        array_free(retired_streams);
        retired_streams = .{};
        array_free(retired_decoders);
        retired_decoders = .{};
        array_free(decoder_windows);
        decoder_windows = .{};
        destroy(*producer_mutex);
        producer_mutex_ready = false;
        destroy(*sound_mutex);
        sound_mutex_ready = false;
    }

    shutdown_decode_queue();
}

#scope_module
audio_fatal :: (message: string) {
    // A caller's assertion callback may return. Never continue unsafe teardown
    // or a failed decode loop, even with optional assertions disabled.
    context.assertion_failed(#caller_location, message);
    exit(1);
}

#scope_export

make_stream :: (data: *Sound_Data, category := Sound_Category.GENERAL_SFX) -> *Sound_Stream {
    push_allocator(context.default_allocator);
    if !data return null;

    lock_player();
    valid := data.loaded && data.nchannels > 0 && data.nchannels <= MAX_CHANNELS_PER_SOUND && data.sampling_rate > 0;
    channels := data.nchannels;
    frames: s64;

    if channels > 0 frames = data.nsamples_times_nchannels / channels;

    unlock_player();
    if !valid return null;
    if cast(u32) category >= MAX_SOUND_CATEGORIES return null;

    stream := New(Sound_Stream);
    stream.sound_data = data;
    stream.sound_name = data.name;
    stream.num_channels = channels;
    stream.category = category;
    stream.internal_flags |= .FIRST_VOLUME_UPDATE;
    if category == .MUSIC stream.user_flags |= .MUSIC;
    if category == .AMBIENCES stream.user_flags |= .AMBIENT;
    if category == .FOOTSTEPS stream.user_flags |= .IS_FOOTSTEP;

    if data.type == .OGG_COMPRESSED {
        stream.decoder = create_ogg_decoder(data, 8000);
        if stream.decoder stream.num_channels = cast(s32) stream.decoder.num_channels;
    } else if data.type == .ADPCM_COMPRESSED {
        adpcm := create_adpcm_decoder(data);
        if adpcm stream.decoder = *adpcm.base;
    } else if data.type != .LINEAR_SAMPLE_ARRAY || !data.samples {
        free(stream);
        return null;
    }

    if data.type != .LINEAR_SAMPLE_ARRAY && !stream.decoder {
        free(stream);
        return null;
    }

    if stream.decoder {
        stream.decoder.stream = stream;
        stream.decoder.add_decode_queue_item = add_decode_queue_item;
        stream.decoder.do_not_queue = stream.decoder.type == .ADPCM || !decode_queue_initted;
        stream.internal_flags |= .WAITING_FOR_INITIAL_DECODER_PAGES;
        stream.samples_needed_to_start_playing = min(stream.decoder.page_size_in_samples, stream.decoder.uncompressed_length_in_samples);
    }

    update_desired_rate(stream);
    stream.current_rate = stream.desired_rate;
    if stream.decoder stream.repeat_end_position = stream.decoder.uncompressed_length_in_samples;
    else stream.repeat_end_position = frames;

    return stream;
}

start_playing :: (stream: *Sound_Stream) {
    push_allocator(context.default_allocator);
    if !stream || !player_initted return;

    lock(*sound_mutex);
    defer unlock(*sound_mutex);
    for retired: retired_streams assert(retired != stream, "cannot restart a retired audio stream");
    if stream.decoder && stream.decoder.type == .OGG stream.decoder.do_not_queue = false;

    stream.marked = true;
    update_desired_rate(stream);
    update_panning(stream);
    prepare_decoder_pages(stream);
    array_add(*active_streams, stream);
}

find :: (entity_id: s64, manager_id: s64) -> *Sound_Stream {
    lock_player();
    defer unlock_player();
    for stream: active_streams if stream.entity_id == entity_id && stream.manager_id == manager_id return stream;
    return null;
}

stop_stream_abruptly :: (id: s64) {
    lock_player();
    index: s64;
    while index < active_streams.count {
        if active_streams[index].entity_id == id retire_stream_at(index);
        else index += 1;
    }

    unlock_player();
    release_retired_streams();
}

stop_stream_abruptly :: (data: *Sound_Data) {
    lock_player();
    index: s64;
    while index < active_streams.count {
        if active_streams[index].sound_data == data retire_stream_at(index);
        else index += 1;
    }

    unlock_player();
    release_retired_streams();
}

stop_all_sounds_abruptly :: (manager_id: s64) {
    lock_player();
    index: s64;
    while index < active_streams.count {
        if active_streams[index].manager_id == manager_id retire_stream_at(index);
        else index += 1;
    }

    unlock_player();
    release_retired_streams();
}

update_desired_rate :: (stream: *Sound_Stream) {
    lock_player();
    defer unlock_player();
    assert(stream && stream.sound_data && is_finite(stream.rate_scale));
    rate := output_sampling_rate();
    if rate == 0 rate = stream.sound_data.sampling_rate;

    stream.desired_rate = cast(float) stream.sound_data.sampling_rate / cast(float) rate * stream.rate_scale;
}

set_repeating :: (stream: *Sound_Stream, repeating: bool) {
    lock_player();
    defer unlock_player();
    if !stream return;

    if repeating {
        stream.user_flags |= .REPEATING;
        if stream.sound_data && stream.sound_data.nchannels > 0
            stream.repeat_end_position = stream.sound_data.nsamples_times_nchannels / stream.sound_data.nchannels;
    } else stream.user_flags &= ~ .REPEATING;
}

pre_entity_update :: () {
    if !sound_mutex_ready return;

    lock_player();
    defer unlock_player();

    for stream: active_streams {
        stream.marked = player_config.keep_sounds_alive_by_default;
        if !(stream.user_flags & .REPEATING) && stream.repeat_end_position >= 0 && stream.play_cursor >= stream.repeat_end_position
            stream.internal_flags |= .FADING_OUT;
        stream.samples_streamed_since_entity_update = 0;
    }
}

post_entity_update :: (dt: float) {
    assert(is_finite(dt) && dt >= 0);
    lock_player();
    index: s64;

    while index < active_streams.count {
        stream := active_streams[index];
        if (stream.internal_flags & .FADING_OUT) && stream.silent_this_frame stream.marked = false;

        update_desired_rate(stream);
        update_panning(stream);
        prepare_decoder_pages(stream);
        if !stream.marked && !(stream.user_flags & .DO_NOT_KILL) retire_stream_at(index);
        else index += 1;
    }

    unlock_player();
    while get_next_completed_decode_queue_item() {}
    if player_initted && (!player_config.update_from_a_thread || backend_needs_async_update_from_main_thread()) update_from_async_thread(false);

    release_retired_streams();
}

update :: (dt := FLOAT32_INFINITY) {
    lock_player();
    now := audio_wall_time();
    elapsed := dt;

    if dt == FLOAT32_INFINITY elapsed = cast(float) max(cast(float64) 0, now - last_entity_time);

    last_entity_time = now;
    unlock_player();
    pre_entity_update();
    post_entity_update(elapsed);
}

fill_sample_buffer :: (buffer: string) {
    assert(buffer.count >= 0 && (buffer.data != null || buffer.count == 0));
    offset: s64;
    while offset < buffer.count {
        lock_player();

        if !player_initted {
            memset(buffer.data + offset, 0, buffer.count - offset);
            unlock_player();
            return;
        }

        channels := output_channel_count();
        frame_bytes := channels * 2;

        assert(buffer.count % frame_bytes == 0);
        frames := min(cast(s64) 256, (buffer.count - offset) / frame_bytes);

        memset(the_accumulator.data, 0, frames * size_of(Audio_Dest_Sample));

        for stream: active_streams {
            prepare_decoder_pages(stream);
            accumulate_stream(stream, the_accumulator, frames);
        }

        for frame: 0..frames - 1 {
            for channel: 0..channels - 1 {
                value := the_accumulator[frame].channels[channel];

                assert(is_finite(value), "nonfinite mixed audio sample");
                record_history_sample(channel, value);
                sample := cast(s16) clamp(value * 32768.0, -32768.0, 32767.0);
                byte_offset := offset + (frame * channels + channel) * 2;
                buffer[byte_offset] = cast,truncate(u8) sample;
                buffer[byte_offset + 1] = cast,truncate(u8) (sample >> 8);
            }

            finish_history_frame();
        }

        // No stream/decoder reference crosses this unlock. Stops can retire and
        // dispose streams between chunks without snapshots or delayed callbacks.
        unlock_player();
        offset += frames * frame_bytes;
    }
}

update_from_async_thread :: (actually_async: bool) {
    lock_producer();

    defer {
        unlock_producer();
        report_backend_notice();
    }

    lock_player();
    running := player_initted && !async_exit;
    alignment := cast(s64) output_channel_count() * 2;

    unlock_player();
    if !running return;

    buffered, minimum := backend_count_buffered_bytes();

    if !backend_can_fill() return;
    if buffered >= minimum return;

    // Only the producer owns native regions. Native polling/copy/commit do not
    // hold the stream lock; rendering acquires it for each bounded chunk.
    chunk := max(cast(s64) 4096, (minimum + 7) / 8);
    chunk = (chunk + alignment - 1) / alignment * alignment;
    bytes := min(chunk, minimum - buffered);
    bytes -= bytes % alignment;
    if bytes == 0 return;

    regions, success := backend_lock_fill_regions(bytes);

    assert(success, "audio output buffer unavailable");
    fill_sample_buffer(regions.buffer0);
    if regions.buffer1.count > 0 fill_sample_buffer(regions.buffer1);

    backend_release_fill_regions(regions);
}

async_audio_update :: (thread: *Thread) -> s64 {
    if player_config.set_async_thread_priority player_config.set_async_thread_priority();

    while true {
        lock(*sound_mutex);
        stopping := async_exit;

        unlock(*sound_mutex);
        if stopping return 0;

        update_from_async_thread(true);
        sleep_milliseconds(2);
    }

    return 0;
}

#scope_module
lock_player :: () {
    if sound_mutex_ready lock(*sound_mutex);
}

unlock_player :: () {
    if sound_mutex_ready unlock(*sound_mutex);
}

lock_producer :: () {
    if producer_mutex_ready lock(*producer_mutex);
    producer_depth += 1;
}

unlock_producer :: () {
    producer_depth -= 1;
    if producer_mutex_ready unlock(*producer_mutex);
}

finish_backend_call :: () {
    flush := producer_depth == 1;
    unlock_producer();
    if flush report_backend_notice();
}

#scope_file
producer_mutex: Mutex;
producer_mutex_ready := false;
producer_depth: s64;
sound_mutex: Mutex;
last_entity_time: float64;
player_config: Sound_Player_Config;
player_initted := false;
sound_mutex_ready := false;
audio_thread: Thread;
audio_thread_started := false;
async_exit := false;
active_streams: [..] *Sound_Stream;
retired_streams: [..] *Sound_Stream;

retire_stream_at :: (index: s64) {
    push_allocator(context.default_allocator);
    stream := active_streams[index];

    for i: index..active_streams.count - 2 active_streams[i] = active_streams[i + 1];

    active_streams.count -= 1;
    for remaining: active_streams if remaining == stream return;

    for i: 0..decoder_windows.count - 1 {
        if decoder_windows[i].stream == stream {
            decoder_windows[i] = decoder_windows[decoder_windows.count - 1];
            decoder_windows.count -= 1;
            break;
        }
    }

    array_add(*retired_streams, stream);
    if stream.decoder array_add(*retired_decoders, stream.decoder);
}

release_retired_streams :: () {
    push_allocator(context.default_allocator);

    while true {
        lock_player();

        if retired_streams.count == 0 {
            unlock_player();
            return;
        }

        stream := retired_streams[retired_streams.count - 1];

        retired_streams.count -= 1;
        unlock_player();

        if stream.decoder {
            decoder := stream.decoder;

            dispose_decoder_resources(decoder);
            lock_player();

            for i: 0..retired_decoders.count - 1 {
                if retired_decoders[i] == decoder {
                    retired_decoders[i] = retired_decoders[retired_decoders.count - 1];
                    retired_decoders.count -= 1;
                    break;
                }
            }

            unlock_player();
            free(decoder);
            stream.decoder = null;
        }

        if player_config.release_asset player_config.release_asset(stream, stream.sound_data);

        free(stream);
    }
}

Decoder_Window :: struct {
    stream: *Sound_Stream;
    decoder: *Cached_Decoder;
    start: s64 = -1;
    end: s64;
    repeat_base: s64 = -1;
}

decoder_windows: [..] Decoder_Window;

prepare_decoder_pages :: (stream: *Sound_Stream) {
    decoder := stream.decoder;
    if !decoder return;

    push_allocator(context.default_allocator);
    cursor := max(cast(s64) 0, cast(s64) stream.play_cursor);
    base := get_base_address(decoder, cursor);
    start := base;

    if stream.current_rate < 0 || stream.desired_rate < 0 start = max(cast(s64) 0, base - decoder.page_size_in_samples);

    end := base + decoder.page_size_in_samples * 3;
    repeat_base: s64 = -1;

    if stream.user_flags & .REPEATING repeat_base = get_base_address(decoder, max(cast(s64) 0, stream.repeat_start_position));

    window: *Decoder_Window;

    for *candidate: decoder_windows if candidate.stream == stream {
        window = candidate;
        break;
    }

    if !window {
        item: Decoder_Window;
        item.stream = stream;
        array_add(*decoder_windows, item);
        window = *decoder_windows[decoder_windows.count - 1];
    }

    if window.decoder != decoder || window.start != start || window.end != end || window.repeat_base != repeat_base {
        unmark_all_pages(decoder);
        mark_relevant_pages(decoder, start, end);
        if repeat_base >= 0 mark_relevant_pages(decoder, repeat_base, repeat_base + decoder.page_size_in_samples);

        free_unmarked_pages(decoder);
        window.decoder = decoder;
        window.start = start;
        window.end = end;
        window.repeat_base = repeat_base;
    }

    if stream.internal_flags & .WAITING_FOR_INITIAL_DECODER_PAGES {
        first := find_page_by_base_address(decoder, base);
        if first && !page_work_pending(first) {
            stream.initial_samples_fetched = first.num_samples_contained;
            if stream.initial_samples_fetched >= stream.samples_needed_to_start_playing stream.internal_flags &= ~ .WAITING_FOR_INITIAL_DECODER_PAGES;
        }
    }
}
