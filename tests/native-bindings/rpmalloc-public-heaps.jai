// Public oracle: reviewed rpmalloc.md heap signatures/contextual records.
// Behavioral assertions cover ownership and real allocation, not SDK internals.
#import "Basic";
RPM :: #import "rpmalloc";
Small :: #import "rpmalloc"(ENABLE_STATISTICS = false);

check_links :: (head: *$T) -> s64 {
    if !head return 0;

    count: s64 = 1;
    tail := head;

    while tail.next {
        next := tail.next;
        assert(next.prev == tail);
        tail = next;
        count += 1;
    }

    assert(head.prev == tail);

    return count;
}

main :: () {
    assert(RPM.initialize() == 0);
    assert(Small.initialize() == 0);
    first := RPM.heap_acquire();
    second := RPM.heap_acquire();

    assert(first != null && second != null && first != second);
    assert(type_info(type_of(first.*)).name == "Heap");
    memory := RPM.heap_alloc(first, 301);

    assert(memory != null);
    assert(RPM.heap_is_this_yours(first, memory));
    assert(!RPM.heap_is_this_yours(first, cast(*void) 123));
    (cast(*u8) memory)[0] = 47;
    small := Small.heap_acquire();

    assert(small != null && type_info(type_of(small.*)).name == "Heap");
    other := Small.heap_alloc(small, 301);

    assert(other != null && !RPM.heap_is_this_yours(first, other));
    assert(!Small.heap_is_this_yours(small, memory));
    small_span_found: bool;

    for 0..125 {
        span := small.size_class[it].partial_span;
        if span && Small.is_in_span(other, span) {
            assert(span.heap == small && span.block_size >= 301);
            small_span_found = true;
        }
    }

    assert(small_span_found);
    Small.heap_free(small, other);
    Small.heap_release(small);
    Small.finalize();
    replacement := RPM.heap_realloc(first, memory, 801, .{});

    assert(replacement != null && (cast(*u8) replacement)[0] == 47);
    assert(RPM.heap_realloc(first, replacement, -1, .{}) == null);
    assert(RPM.heap_realloc(first, replacement, 0x4000_0000_0000_0000, .{}) == null);
    assert((cast(*u8) replacement)[0] == 47);
    span_found: bool;

    for 0..125 {
        span := first.size_class[it].partial_span;
        if span && RPM.is_in_span(replacement, span) {
            assert(type_info(type_of(span.*)).name == "Span");
            assert(span.heap == first && span.block_count > 0 && span.block_size >= 801);
            tail := span;
            while tail.next tail = tail.next;

            assert(span.prev == tail);
            assert(span.free_list_limit <= span.block_count);
            assert(!RPM.is_in_span(cast(*void) 123, span));
            span_found = true;
        }
    }

    assert(span_found);
    // Occupy enough blocks to exercise multi-element full and partial lists.
    blocks: [64] *void;
    for *blocks {
        it.* = RPM.heap_alloc(first, 6000);
        assert(it.* != null && RPM.heap_is_this_yours(first, it.*));
    }

    full_spans: s64;

    for 0..125 {
        check_links(first.size_class[it].partial_span);
        full_spans += check_links(first.full_span[it]);
    }

    assert(full_spans >= 2);
    for blocks RPM.heap_free(first, it);

    current_allocations: s64;
    for 0..126 current_allocations += first.size_class_use[it].alloc_current;

    assert(current_allocations > 0);
    RPM.heap_free(first, replacement);
    memory = RPM.heap_aligned_alloc(second, 64, 321);
    assert(memory != null && (cast(u64) memory & 63) == 0);
    (cast(*u8) memory)[0] = 91;
    memory = RPM.heap_aligned_realloc(second, memory, 64, 1024, .{});
    assert(memory != null && (cast(*u8) memory)[0] == 91);
    RPM.heap_free_all(second);
    cached_memory := RPM.heap_alloc(second, 100_000);

    assert(cached_memory != null && RPM.heap_is_this_yours(second, cached_memory));
    RPM.heap_free(second, cached_memory);
    // Query only: never dereference the former allocation. Cache/reserve-only
    // ranges are excluded by the reviewed active-list ownership contract.
    assert(!RPM.heap_is_this_yours(second, cached_memory));
    memory = RPM.heap_alloc(second, 4096);
    assert(memory != null);
    RPM.heap_free(second, memory);
    RPM.heap_release(second);
    RPM.heap_thread_set_current(first);
    memory = RPM.rpmalloc(512);
    assert(memory != null && RPM.heap_is_this_yours(first, memory));
    (cast(*u8) memory)[0] = 93;
    current := RPM.heap_acquire();

    assert(current != null);
    RPM.heap_thread_set_current(current);
    // Replacing the current heap ends first's handle lifetime. Its outstanding
    // allocation remains freeable through the matching global family.
    assert((cast(*u8) memory)[0] == 93);
    RPM.rpfree(memory);
    memory = RPM.rpmalloc(512);
    assert(memory != null && RPM.heap_is_this_yours(current, memory));
    RPM.rpfree(memory);
    RPM.heap_thread_set_current(current);
    assert(RPM.is_thread_initialized());
    RPM.heap_release(current);
    RPM.thread_initialize();
    RPM.finalize();
}
