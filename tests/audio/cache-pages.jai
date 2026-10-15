#import "Basic";
#import "Sound_Player";

main :: () {
    encoded: [48] u8;
    // Six mono IMA blocks, each with a zero predictor/index and eight zero nibbles.
    data: Sound_Data;
    data.loaded = true;
    data.type = .ADPCM_COMPRESSED;
    data.buffer = .{ encoded.count, encoded.data };
    data.nchannels = 1;
    data.nBlockAlign = 8;
    data.wSamplesPerBlock = 9;
    data.nsamples_times_nchannels = 54;
    decoder := create_adpcm_decoder(*data);

    assert(decoder && decoder.page_size_in_samples == 27);
    assert(get_base_address(*decoder.base, 28) == 27);
    mark_relevant_pages(*decoder.base, 2, 30);
    assert(decoder.pages.count == 2);
    assert(decoder.issued_start_addresses.count == 0, "synchronous ADPCM must have no outstanding work");
    found, start, end, memory := find_page_containing_cursor(*decoder.base, 29);

    assert(found && start == 27 && end == 54 && memory);
    for i: 0..26 assert((cast(*s16) memory)[i] == 0);

    unmark_all_pages(*decoder.base);
    mark_relevant_pages(*decoder.base, 27, 54);
    free_unmarked_pages(*decoder.base);
    assert(decoder.pages.count == 1 && decoder.pages[0].start_address == 27);
    assert(!find_page_by_base_address(*decoder.base, 0));
    deinit(*decoder.base);
    print("ok\n");
}
