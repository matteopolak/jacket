ADPCM_BLOCKS_PER_PAGE :: 3;

create_adpcm_decoder :: (data: *Sound_Data) -> *Adpcm_Decoder {
    push_allocator(context.default_allocator);
    if !valid_adpcm(data) return null;

    decoder := New(Adpcm_Decoder);
    decoder.type = .ADPCM;
    decoder.sound_data = data;
    decoder.num_channels = data.nchannels;
    decoder.sampling_rate = data.sampling_rate;
    decoder.uncompressed_length_in_samples = data.nsamples_times_nchannels / data.nchannels;
    decoder.page_size_in_samples = data.wSamplesPerBlock * ADPCM_BLOCKS_PER_PAGE;
    decoder.make_page = make_page_adpcm;
    decoder.recompute_page = recompute_page_adpcm;

    return decoder;
}

make_page_adpcm :: (using decoder: *Cached_Decoder) -> *Cached_Decoder_Page {
    push_allocator(context.default_allocator);
    page := New(Cached_Decoder_Page);
    page.owner = decoder;
    page.type = .ADPCM;
    page.buffer_length_in_bytes = page_size_in_samples * num_channels * 2;
    page.data = alloc(page.buffer_length_in_bytes);
    assert(page.data != null);

    return page;
}

recompute_page_adpcm :: (using decoder: *Cached_Decoder, page: *Cached_Decoder_Page) {
    // Factory validation already checked every block header. Borrowed source
    // bytes must stay stable; per-page work validates only this page's bounds.
    assert(sound_data && sound_data.nBlockAlign > 0 && sound_data.wSamplesPerBlock > 0);
    assert(page.owner == decoder && page.start_address >= 0 && page.start_address < uncompressed_length_in_samples && page.start_address % sound_data.wSamplesPerBlock == 0);
    offset := page.start_address / sound_data.wSamplesPerBlock * sound_data.nBlockAlign;
    payload := adpcm_payload(sound_data);
    bytes := min(payload.count - offset, cast(s64) sound_data.nBlockAlign * ADPCM_BLOCKS_PER_PAGE);

    if offset < 0 || bytes <= 0 || bytes % sound_data.nBlockAlign != 0 audio_fatal("ADPCM page lies outside complete compressed blocks");

    input: [] u8 = .{ bytes, payload.data + offset };
    output: [] u8 = .{ page.buffer_length_in_bytes, cast(*u8) page.data };
    info: A.Adpcm_Info = .{ cast(s32) num_channels, sound_data.nBlockAlign, sound_data.wSamplesPerBlock };
    decoded := A.IMA_ADPCM_decode(info, input, output);

    if decoded <= 0 audio_fatal("ADPCM page decoder made no progress");

    page.num_samples_contained = min(cast(s64) decoded / (num_channels * 2), uncompressed_length_in_samples - page.start_address);
    page.end_address = page.start_address + page.num_samples_contained;
}

#scope_module
valid_adpcm :: (data: *Sound_Data) -> bool {
    if !data || data.type != .ADPCM_COMPRESSED || data.nchannels == 0 || data.nchannels > MAX_CHANNELS_PER_SOUND return false;

    channels := cast(s64) data.nchannels;
    if data.nBlockAlign < channels * 4 || data.wSamplesPerBlock <= 0 || data.sampling_rate == 0 return false;

    payload := data.nBlockAlign - channels * 4;
    if payload % (channels * 4) != 0 || data.wSamplesPerBlock != 1 + payload * 2 / channels return false;

    input := adpcm_payload(data);
    if input.count == 0 || input.count % data.nBlockAlign != 0 return false;
    if data.nsamples_times_nchannels < 0 || data.nsamples_times_nchannels / channels > input.count / data.nBlockAlign * data.wSamplesPerBlock return false;

    for block: 0..input.count / data.nBlockAlign - 1 {
        for channel: 0..channels - 1 {
            if input[block * data.nBlockAlign + channel * 4 + 2] > 88 return false;
        }
    }

    return true;
}

adpcm_payload :: (data: *Sound_Data) -> string {
    if data.buffer.count >= 4 && get_magic4(data.buffer, 0) == 0x46464952 {
        _, samples, success := Wav.get_wav_header(data.buffer);
        if success return samples;
        return .{};
    }

    return data.buffer;
}
