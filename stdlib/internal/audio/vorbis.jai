get_compressed_rate_channels_and_size :: (s: string) -> (num_samples: s64, num_channels: s64, sampling_rate: s64) {
    if s.count < 4 || s.count > 0x7fffffff return 0, 0, 0;

    error: s32;
    file := V.stb_vorbis_open_memory(s.data, cast(s32) s.count, *error, null);

    if !file return 0, 0, 0;

    defer V.stb_vorbis_close(file);
    info := V.stb_vorbis_get_info(file);
    if info.channels <= 0 || info.channels > MAX_CHANNELS_PER_SOUND || info.sample_rate == 0 return 0, 0, 0;

    length := V.stb_vorbis_stream_length_in_samples(file);

    return length, info.channels, info.sample_rate;
}

start_vorbis_file :: (using decoder: *Cached_Ogg_Decoder) -> bool {
    if vorbis_file return true;
    if !sound_data || sound_data.buffer.count > 0x7fffffff || sound_data.buffer.count < 4 return false;

    error: s32;

    vorbis_file = V.stb_vorbis_open_memory(sound_data.buffer.data, cast(s32) sound_data.buffer.count, *error, null);
    if !vorbis_file return false;

    info := V.stb_vorbis_get_info(vorbis_file);
    if info.channels != num_channels || info.sample_rate != sampling_rate {
        V.stb_vorbis_close(vorbis_file);
        vorbis_file = null;
        return false;
    }

    return true;
}

create_ogg_decoder :: (data: *Sound_Data, _page_size_in_samples: s64) -> *Cached_Decoder {
    push_allocator(context.default_allocator);
    if !data || data.type != .OGG_COMPRESSED || _page_size_in_samples <= 0 return null;

    frames, channels, rate := get_compressed_rate_channels_and_size(data.buffer);
    if frames <= 0 || channels <= 0 || channels > MAX_CHANNELS_PER_SOUND || rate <= 0 return null;
    if _page_size_in_samples > 0x7fffffff / (channels * 2) return null;

    decoder := New(Cached_Ogg_Decoder);
    decoder.type = .OGG;
    decoder.sound_data = data;
    decoder.num_channels = channels;
    decoder.sampling_rate = rate;
    decoder.uncompressed_length_in_samples = frames;
    decoder.page_size_in_samples = _page_size_in_samples;
    decoder.bytes_per_mono_sample = 2;
    decoder.uncompressed_length_in_bytes = frames * channels * 2;
    decoder.make_page = make_page_ogg;
    decoder.recompute_page = recompute_page_ogg;
    decoder.deinit_proc = deinit_proc_ogg;
    if !start_vorbis_file(decoder) {
        free(decoder);
        return null;
    }

    // Codec setup owns independent handles and runs outside the mixer lock.
    // Publish shared Sound_Data metadata together under the reader's lock.
    lock_player();
    data.nchannels = cast(u16) channels;
    data.sampling_rate = cast(u32) rate;
    data.nsamples_times_nchannels = frames * channels;
    assign_channel_directions(data);
    unlock_player();

    return *decoder.base;
}

deinit_proc_ogg :: (decoder: *Cached_Decoder) {
    ogg := cast(*Cached_Ogg_Decoder) decoder;
    if ogg.vorbis_file {
        V.stb_vorbis_close(ogg.vorbis_file);
        ogg.vorbis_file = null;
    }
}

make_page_ogg :: (using decoder: *Cached_Decoder) -> *Cached_Decoder_Page {
    push_allocator(context.default_allocator);
    page := New(Cached_Decoder_Page);
    page.type = .OGG;
    page.owner = decoder;
    page.buffer_length_in_bytes = page_size_in_samples * num_channels * 2;
    page.data = alloc(page.buffer_length_in_bytes);
    assert(page.data != null);

    return page;
}

recompute_page_ogg :: (using decoder: *Cached_Decoder, _page: *Cached_Decoder_Page) {
    ogg := cast(*Cached_Ogg_Decoder) decoder;

    assert(_page.owner == decoder && _page.start_address >= 0 && _page.start_address < uncompressed_length_in_samples);
    opened := start_vorbis_file(ogg);
    if !opened audio_fatal("Vorbis decoder could not open its stream");

    assert(_page.start_address <= 0xffffffff);
    sought := V.stb_vorbis_seek(ogg.vorbis_file, cast(u32) _page.start_address);
    if sought == 0 audio_fatal("Vorbis page seek failed");

    frames := min(page_size_in_samples, uncompressed_length_in_samples - _page.start_address);
    decoded: s64;

    while decoded < frames {
        count := V.stb_vorbis_get_samples_short_interleaved(ogg.vorbis_file, cast(s32) num_channels, (cast(*s16) _page.data) + decoded * num_channels, cast(s32) ((frames - decoded) * num_channels));
        if count <= 0 audio_fatal("Vorbis page ended before its declared sample length");
        decoded += count;
    }

    _page.num_samples_contained = decoded;
    _page.end_address = _page.start_address + decoded;
}
