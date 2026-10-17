#import "Basic";
#import "Sound_Player";

computed: int;

make_page :: (decoder: *Cached_Decoder) -> *Cached_Decoder_Page {
    push_allocator(context.default_allocator);
    page := New(Cached_Decoder_Page);
    page.owner = decoder;
    page.data = alloc(16);
    page.buffer_length_in_bytes = 16;

    return page;
}

compute :: (decoder: *Cached_Decoder, page: *Cached_Decoder_Page) {
    sleep_milliseconds(1);
    memset(page.data, 0, 16);
    page.num_samples_contained = 8;
    computed += 1;
}

main :: () {
    init_sound_player_decode_queue();
    decoder := New(Cached_Decoder);
    decoder.page_size_in_samples = 8;
    decoder.uncompressed_length_in_samples = 128;
    decoder.num_channels = 1;
    decoder.make_page = make_page;
    decoder.recompute_page = compute;
    decoder.add_decode_queue_item = add_decode_queue_item;
    decoder.do_not_queue = false;
    mark_relevant_pages(decoder, 0, 128);
    assert(decoder.pages.count == 16);
    deinit(decoder);
    assert(computed == 16, "deinit must drain all issued work instead of cancelling it");
    assert(get_next_completed_decode_queue_item() == null);
    sound_player_shutdown();
    print("ok\n");
}
