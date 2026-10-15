#import "Basic"(ENABLE_ASSERT = false);
#import "File";
#import "Sound_Player";

check :: (condition: bool) {
    if !condition {
        print("assertion-independent check failed\n", to_standard_error = true);
        exit(1);
    }
}

computed: int;

must_not_run :: () -> bool {
    exit(2);
    return false;
}

compute :: (decoder: *Cached_Decoder, page: *Cached_Decoder_Page) {
    sleep_milliseconds(1);
    computed += 1;
    page.num_samples_contained = 1;
}

main :: () {
    assert(must_not_run());
    bytes, ok := read_entire_file("fixtures/tone.ogg");

    check(ok);
    defer free(bytes);
    data: Sound_Data;
    data.type = .OGG_COMPRESSED;
    data.buffer = bytes;
    baseline := create_ogg_decoder(*data, 1024);
    reordered := create_ogg_decoder(*data, 1024);

    check(baseline != null && reordered != null);
    mark_relevant_pages(baseline, 0, 1024);
    mark_relevant_pages(reordered, 2048, 3072);
    mark_relevant_pages(reordered, 0, 1024);
    first, first_start, first_end, first_data := find_page_containing_cursor(baseline, 0);
    actual, actual_start, actual_end, actual_data := find_page_containing_cursor(reordered, 0);

    check(first && actual);
    check(first_start == 0 && actual_start == 0 && first_end == 1024 && actual_end == 1024);
    expected_samples := cast(*s16) first_data;
    actual_samples := cast(*s16) actual_data;

    for i: 0..2047 check(expected_samples[i] == actual_samples[i]);

    deinit(baseline);
    deinit(reordered);

    init_sound_player_decode_queue();
    decoder: Cached_Decoder;
    decoder.recompute_page = compute;
    pages: [32] Cached_Decoder_Page;
    for *page: pages {
        page.owner = *decoder;
        page.start_address = it_index;
        add_decode_queue_item(page);
    }

    sound_player_shutdown();
    check(!decode_queue_initted && computed == pages.count);
    check(decoder.issued_start_addresses.count == 0);
    array_free(decoder.issued_start_addresses);
    print("ok\n");
}
