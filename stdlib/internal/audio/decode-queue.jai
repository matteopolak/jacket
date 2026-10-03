decode_queue_initted := false;

#if CACHED_OGG_DECODE_THREADED {
    decode_queue: [..] *Cached_Decoder_Page;
    decode_queue_semaphore: Semaphore;
    decode_queue_lock: Mutex;
    decode_queue_thread: Thread;
}

completed_queue: [..] *Cached_Decoder_Page;
completed_queue_lock: Mutex;

init_sound_player_decode_queue :: () {
    push_allocator(context.default_allocator);
    if decode_queue_initted return;

    init(*decode_queue_lock);
    init(*completed_queue_lock);
    init(*decode_queue_semaphore);
    init(*decode_idle);
    decode_stopping = false;

    if !thread_init(*decode_queue_thread, decode_worker, name = "audio decode") {
        destroy(*decode_idle);
        destroy(*decode_queue_semaphore);
        destroy(*completed_queue_lock);
        destroy(*decode_queue_lock);
        audio_fatal("audio decode worker creation failed");

        return;
    }

    decode_queue_initted = true;
    thread_start(*decode_queue_thread);
}

process_one_page :: (page: *Cached_Decoder_Page) {
    push_allocator(context.default_allocator);
    assert(page && page.owner && page.owner.recompute_page);
    page.owner.recompute_page(page.owner, page);
    if !decode_queue_initted return;

    lock(*decode_queue_lock);
    remove_issued_address(page.owner, page.start_address);
    lock(*completed_queue_lock);
    array_add(*completed_queue, page);
    unlock(*completed_queue_lock);
    unlock(*decode_queue_lock);
}

add_decode_queue_item :: (page: *Cached_Decoder_Page) {
    push_allocator(context.default_allocator);
    assert(decode_queue_initted, "initialize the audio decode queue before submitting work");
    assert(page && page.owner);
    lock(*decode_queue_lock);
    defer unlock(*decode_queue_lock);
    assert(!decode_stopping, "cannot queue work during audio shutdown");
    if decoding_page == page return;
    for queued: decode_queue if queued == page return;

    lock(*completed_queue_lock);
    already_completed := false;
    for completed: completed_queue if completed == page already_completed = true;

    unlock(*completed_queue_lock);
    if already_completed return;

    array_add(*page.owner.issued_start_addresses, page.start_address);
    array_add(*decode_queue, page);
    signal(*decode_queue_semaphore);
}

get_next_completed_decode_queue_item :: () -> *Cached_Decoder_Page {
    if !decode_queue_initted return null;

    lock(*completed_queue_lock);
    defer unlock(*completed_queue_lock);
    if completed_queue.count == 0 return null;

    result := completed_queue[0];

    for i: 0..completed_queue.count - 2 completed_queue[i] = completed_queue[i + 1];

    completed_queue.count -= 1;

    return result;
}

#scope_module
page_work_pending :: (page: *Cached_Decoder_Page) -> bool {
    if !decode_queue_initted return false;

    lock(*decode_queue_lock);
    defer unlock(*decode_queue_lock);
    if decoding_page == page return true;
    for pending: decode_queue if pending == page return true;

    return false;
}

forget_page_work :: (page: *Cached_Decoder_Page) {
    if !decode_queue_initted return;

    lock(*decode_queue_lock);
    defer unlock(*decode_queue_lock);
    while decoding_page == page || page_is_queued(page) wait(*decode_idle, *decode_queue_lock);

    lock(*completed_queue_lock);
    remove_page_pointer(*completed_queue, page);
    unlock(*completed_queue_lock);
}

shutdown_decode_queue :: () {
    push_allocator(context.default_allocator);
    if !decode_queue_initted return;

    lock(*decode_queue_lock);
    decode_stopping = true;
    unlock(*decode_queue_lock);
    signal(*decode_queue_semaphore);
    joined := thread_is_done(*decode_queue_thread, -1);
    if !joined audio_fatal("audio decode worker join failed");

    thread_deinit(*decode_queue_thread);
    decode_queue_initted = false;
    array_free(decode_queue);
    array_free(completed_queue);
    decode_queue = .{};
    completed_queue = .{};
    destroy(*decode_idle);
    destroy(*decode_queue_semaphore);
    destroy(*completed_queue_lock);
    destroy(*decode_queue_lock);
}

#scope_file
decode_idle: Condition_Variable;
decode_stopping := false;
decoding_page: *Cached_Decoder_Page;

remove_page_pointer :: (queue: *[..] *Cached_Decoder_Page, page: *Cached_Decoder_Page) {
    i: s64;
    while i < queue.count {
        if queue.data[i] != page {
            i += 1;
            continue;
        }

        for j: i..queue.count - 2 queue.data[j] = queue.data[j + 1];

        queue.count -= 1;
    }
}

decode_worker :: (thread: *Thread) -> s64 {
    while true {
        waited := wait_for(*decode_queue_semaphore);
        if waited != .SUCCESS audio_fatal("audio decode wait failed");

        lock(*decode_queue_lock);

        if decode_queue.count == 0 {
            stopping := decode_stopping;
            unlock(*decode_queue_lock);
            if stopping return 0;
            continue;
        }

        page := decode_queue[0];

        for i: 0..decode_queue.count - 2 decode_queue[i] = decode_queue[i + 1];

        decode_queue.count -= 1;
        decoding_page = page;
        unlock(*decode_queue_lock);
        process_one_page(page);
        lock(*decode_queue_lock);
        decoding_page = null;
        wake_all(*decode_idle);
        stopping := decode_stopping && decode_queue.count == 0;

        unlock(*decode_queue_lock);
        if stopping return 0;
    }

    return 0;
}

page_is_queued :: (page: *Cached_Decoder_Page) -> bool {
    for queued: decode_queue if queued == page return true;
    return false;
}

remove_issued_address :: (decoder: *Cached_Decoder, address: s64) {
    for i: 0..decoder.issued_start_addresses.count - 1 {
        if decoder.issued_start_addresses[i] == address {
            for j: i..decoder.issued_start_addresses.count - 2 decoder.issued_start_addresses[j] = decoder.issued_start_addresses[j + 1];
            decoder.issued_start_addresses.count -= 1;
            return;
        }
    }
}
