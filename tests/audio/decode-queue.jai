#import "Basic";
#import "Sound_Player";

computed: int;

compute :: (decoder: *Cached_Decoder, page: *Cached_Decoder_Page) {
    computed += 1;
    page.num_samples_contained = 7;
}

main :: () {
    init_sound_player_decode_queue();
    decoder: Cached_Decoder;
    decoder.recompute_page = compute;
    page: Cached_Decoder_Page;
    page.owner = *decoder;
    add_decode_queue_item(*page);
    completed: *Cached_Decoder_Page;

    for i: 0..999 {
        completed = get_next_completed_decode_queue_item();
        if completed break;
        sleep_milliseconds(1);
    }

    assert(completed == *page && page.num_samples_contained == 7 && computed == 1);
    assert(!get_next_completed_decode_queue_item());
    sound_player_shutdown();
    assert(!decode_queue_initted);
    print("ok\n");
}
