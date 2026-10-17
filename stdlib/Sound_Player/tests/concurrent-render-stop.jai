#import "Basic";
#import "Thread";
#import "Sound_Player";

ready: Semaphore;
attempting: Semaphore;
trigger := false;
released: int;
released_after_frames: s64;

allocate_page :: (decoder: *Cached_Decoder) -> *Cached_Decoder_Page {
    push_allocator(context.default_allocator);
    page := New(Cached_Decoder_Page);
    page.owner = decoder;
    page.buffer_length_in_bytes = decoder.page_size_in_samples * 2;
    page.data = alloc(page.buffer_length_in_bytes);

    return page;
}

compute :: (decoder: *Cached_Decoder, page: *Cached_Decoder_Page) {
    memset(page.data, 0, page.buffer_length_in_bytes);
    page.num_samples_contained = min(decoder.page_size_in_samples, decoder.uncompressed_length_in_samples - page.start_address);
    page.end_address = page.start_address + page.num_samples_contained;

    if trigger {
        trigger = false;
        signal(*ready);
        assert(wait_for(*attempting) == .SUCCESS);
    }
}

stop_worker :: (thread: *Thread) -> s64 {
    assert(wait_for(*ready) == .SUCCESS);
    signal(*attempting);
    stop_stream_abruptly(88);
    return 0;
}

release :: (stream: *Sound_Stream, data: *Sound_Data) {
    assert(find(88, 0) == null);
    released_after_frames = stream.samples_streamed_since_entity_update;
    released += 1;
    set_master_volume(0);
    assert(count_retired_decoders() == 0);
}

dispose :: (decoder: *Cached_Decoder) {
    assert(count_retired_decoders() == 1, "decoder stays published and alive while resources drain");
    assert(decoder.issued_start_addresses.count == 0);
    set_master_volume(0);
}

main :: () {
    config: Sound_Player_Config;
    config.update_from_a_thread = false;
    config.release_asset = release;
    set_master_volume(0);

    if !sound_player_init(config) {
        sound_player_shutdown();
        print("native-device-unavailable\nok\n");
        return;
    }

    print("native-device-opened\n");
    init(*ready);
    init(*attempting);
    data: Sound_Data;
    data.loaded = true;
    data.type = .ADPCM_COMPRESSED;
    data.sampling_rate = cast(u32) get_audio_sampling_rate();
    data.nsamples_times_nchannels = 65536;
    decoder := New(Cached_Decoder);
    decoder.type = .ADPCM;
    decoder.page_size_in_samples = 256;
    decoder.uncompressed_length_in_samples = 65536;
    decoder.num_channels = 1;
    decoder.sound_data = *data;
    decoder.make_page = allocate_page;
    decoder.recompute_page = compute;
    decoder.deinit_proc = dispose;
    stream := New(Sound_Stream);
    stream.num_channels = 1;
    stream.entity_id = 88;
    stream.sound_data = *data;
    stream.decoder = decoder;
    stream.repeat_end_position = 65536;
    decoder.stream = stream;
    start_playing(stream);
    worker: Thread;

    assert(thread_init(*worker, stop_worker));
    thread_start(*worker);
    trigger = true;
    // An owned synchronous page callback coordinates a stop during rendering.
    // No scheduling deadline, mutex fairness or before-end timing is asserted.
    output := NewArray(16384 * 8 * 2, u8);

    fill_sample_buffer(.{ output.count, output.data });
    assert(thread_is_done(*worker, -1));
    thread_deinit(*worker);
    assert(released == 1 && released_after_frames > 0);
    for sample: output assert(sample == 0);

    free(output.data);
    sound_player_shutdown();
    assert(!decode_queue_initted);
    destroy(*ready);
    destroy(*attempting);
    print("ok\n");
}
