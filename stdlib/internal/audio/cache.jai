retired_decoders: [..] *Cached_Decoder;
SECONDS_TO_FETCH_OGG :: 0.4;
SECONDS_TO_FETCH_ADPCM :: 0.4;
CACHED_OGG_DECODE_THREADED :: true;
CACHED_OGG_DECODE_NUM_THREADS :: 1;

count_retired_decoders :: () -> s64 {
    lock_player();
    defer unlock_player();
    return retired_decoders.count;
}

get_base_address :: (decoder: *Cached_Decoder, play_cursor: s64) -> s64 {
    assert(decoder && decoder.page_size_in_samples > 0 && play_cursor >= 0);
    return play_cursor / decoder.page_size_in_samples * decoder.page_size_in_samples;
}

unmark_all_pages :: (using decoder: *Cached_Decoder) {
    for page: pages page.marked = false;
}

mark_relevant_pages :: (using decoder: *Cached_Decoder, start: s64, end: s64) {
    push_allocator(context.default_allocator);
    assert(page_size_in_samples > 0 && make_page && recompute_page);
    first := max(cast(s64) 0, start);
    limit := min(end, uncompressed_length_in_samples);

    if limit <= first return;

    address := get_base_address(decoder, first);
    while address < limit {
        page := find_page_by_base_address(decoder, address);
        if !page {
            page = make_page(decoder);
            assert(page != null, "audio page allocation failed");
            page.owner = decoder;
            page.type = type;
            page.start_address = address;
            page.end_address = min(address + page_size_in_samples, uncompressed_length_in_samples);
            page.page_index = address / page_size_in_samples;
            array_add(*pages, page);
            page.marked = true;
            if !do_not_queue && add_decode_queue_item add_decode_queue_item(page);
            else recompute_page(decoder, page);
        } else page.marked = true;

        address += page_size_in_samples;
    }
}

free_unmarked_pages :: (using decoder: *Cached_Decoder) {
    push_allocator(context.default_allocator);
    index: s64;
    while index < pages.count {
        page := pages[index];
        if page.marked || page_work_pending(page) {
            index += 1;
            continue;
        }

        forget_page_work(page);
        free(page.data);
        free(page);
        for i: index..pages.count - 2 pages[i] = pages[i + 1];

        pages.count -= 1;
    }
} @Releases

find_page_by_base_address :: (using decoder: *Cached_Decoder, start_address: s64) -> *Cached_Decoder_Page {
    for page: pages if page.start_address == start_address return page;
    return null;
}

find_page_containing_cursor :: (using decoder: *Cached_Decoder, play_cursor: s64) -> (found: bool, start_address: s64, end_address: s64, data: *void) {
    for page: pages {
        if page_work_pending(page) continue;
        if play_cursor >= page.start_address && play_cursor < page.start_address + page.num_samples_contained && page.data
            return true, page.start_address, page.start_address + page.num_samples_contained, page.data;
    }

    return false, 0, 0, null;
}

deinit :: (using decoder: *Cached_Decoder) {
    push_allocator(context.default_allocator);
    if !decoder return;
    dispose_decoder_resources(decoder);
    free(decoder);
} @Releases

#scope_module
dispose_decoder_resources :: (using decoder: *Cached_Decoder) {
    push_allocator(context.default_allocator);
    for page: pages forget_page_work(page);
    if deinit_proc deinit_proc(decoder);

    for page: pages {
        free(page.data);
        free(page);
    }

    array_free(pages);
    array_free(issued_start_addresses);
    pages = .{};
    issued_start_addresses = .{};
}
