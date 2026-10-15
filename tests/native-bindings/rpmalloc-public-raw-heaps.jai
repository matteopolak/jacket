// Reviewed rpmalloc.md raw first-class heap profile. This program requires the
// explicitly selected private build profile, not a new public import option.
#import "Basic";
Raw :: #import "rpmalloc"(USE_RAW = true);
Small :: #import "rpmalloc"(USE_RAW = true, ENABLE_STATISTICS = false);
Rewrite :: #import "rpmalloc";

main :: () {
    #assert Raw.Heap == Raw.rpmalloc_heap_t;
    #assert type_info(Raw.Heap).name == "heap_t";
    #assert Raw.heap_acquire == Raw.rpmalloc_heap_acquire;
    #assert Raw.heap_alloc == Raw.rpmalloc_heap_alloc;
    #assert Raw.heap_release == Raw.rpmalloc_heap_release;
    #assert Raw.heap_aligned_alloc == Raw.rpmalloc_heap_aligned_alloc;
    #assert Raw.heap_realloc == Raw.rpmalloc_heap_realloc;
    #assert Raw.heap_aligned_realloc == Raw.rpmalloc_heap_aligned_realloc;
    #assert Raw.heap_free == Raw.rpmalloc_heap_free;
    #assert Raw.heap_free_all == Raw.rpmalloc_heap_free_all;
    #assert Raw.heap_thread_set_current == Raw.rpmalloc_heap_thread_set_current;
    assert(Raw.initialize() == 0 && Small.initialize() == 0);
    assert(Rewrite.initialize() == 0);
    heap := Raw.heap_acquire();
    small := Small.heap_acquire();

    assert(heap != null && small != null);
    assert(type_info(type_of(heap.*)).name == "heap_t");
    blocks := Raw.rpmalloc_heap_calloc(heap, 7, 43);

    assert(blocks != null);
    for 0..300 assert((cast(*u8) blocks)[it] == 0);

    (cast(*u8) blocks)[0] = 71;
    replacement := Raw.heap_realloc(heap, blocks, 900, .{});

    assert(replacement != null && (cast(*u8) replacement)[0] == 71);
    assert(Raw.rpmalloc_heap_calloc(heap, 0xffff_ffff_ffff_ffff, 2) == null);
    assert(Raw.heap_realloc(heap, replacement, 0xffff_ffff_ffff_ffff, .{}) == null);
    assert((cast(*u8) replacement)[0] == 71);
    Raw.heap_free(heap, replacement);
    aligned := Raw.rpmalloc_heap_aligned_calloc(heap, 64, 7, 43);

    assert(aligned != null && (cast(u64) aligned & 63) == 0);
    for 0..300 assert((cast(*u8) aligned)[it] == 0);

    Raw.heap_free(heap, aligned);
    other := Small.heap_aligned_alloc(small, 64, 301);

    assert(other != null && (cast(u64) other & 63) == 0);
    (cast(*u8) other)[0] = 61;
    other = Small.heap_aligned_realloc(small, other, 64, 1024, .{});
    assert(other != null && (cast(*u8) other)[0] == 61);
    Small.heap_free_all(small);
    Small.heap_release(small);
    Small.finalize();
    Raw.heap_thread_set_current(heap);
    blocks = Raw.rpmalloc(700);
    assert(blocks != null);
    Raw.rpfree(blocks);
    Raw.heap_release(heap);
    Raw.finalize();
    memory := Rewrite.rpmalloc(512);

    assert(memory != null);
    Rewrite.rpfree(memory);
    Rewrite.finalize();
}
