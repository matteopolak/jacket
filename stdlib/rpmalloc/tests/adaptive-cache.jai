// Reviewed conditional public fields and actual private native accounting.
RPM :: #import "rpmalloc"(ENABLE_STATISTICS = false, ENABLE_ADAPTIVE_THREAD_CACHE = true);
Stats :: #import "rpmalloc"(ENABLE_ADAPTIVE_THREAD_CACHE = true);
Raw :: #import "rpmalloc"(USE_RAW = true, ENABLE_STATISTICS = false, ENABLE_ADAPTIVE_THREAD_CACHE = true);
Bridge :: #import "internal/bindings/rpmalloc-heaps"(STATISTICS = false, ADAPTIVE = true);
Domain :: #import "internal/bindings/rpmalloc-domain"(Owner = RPM.rpmalloc_config_t, STATISTICS = false, RAW = false, PUBLIC_HEAPS = true, GLOBAL_CACHE = true, THREAD_CACHE = true, ASSERTS = true, VALIDATE_ARGS = true, ADAPTIVE = true);

probe :: #library,system "rpmalloc_adaptive_probe";
native_current :: (heap: *void, index: u32) -> s32 #foreign probe;
native_high :: (heap: *void, index: u32) -> s32 #foreign probe;
libc :: #library,system "c";
abort :: () #foreign libc;

check :: (condition: bool) {
    if !condition abort();
}

main :: () {
    check(RPM.initialize() == 0);
    heap := RPM.heap_acquire();

    check(heap != null);
    check(size_of(type_of(heap.span_use[0])) == 8);
    check(size_of(type_of(heap.*)) == Bridge.layout_size(6));
    slots: [31] *void;
    for *slots {
        it.* = RPM.heap_alloc(heap, 4096);
        check(it.* != null);
        (cast(*u8) it.*)[0] = 71;
    }

    previous := Domain.select();
    native := cast(*void) Bridge.native_heap(heap);

    Domain.restore(previous);

    check(native != cast(*void) heap);
    check(heap.span_use[0].current > 0);
    check(heap.span_use[0].high >= heap.span_use[0].current);

    for 0..62 {
        check(heap.span_use[it].current == native_current(native, cast(u32) it));
        check(heap.span_use[it].high == native_high(native, cast(u32) it));
    }

    for slots {
        check((cast(*u8) it)[0] == 71);
        RPM.heap_free(heap, it);
    }

    RPM.heap_free_all(heap);
    for 0..62 check(heap.span_use[it].current == native_current(native, cast(u32) it));

    RPM.heap_release(heap);
    RPM.finalize();
    check(Stats.initialize() == 0);
    stats_heap := Stats.heap_acquire();
    stats_memory := Stats.heap_alloc(stats_heap, 4096);

    check(stats_memory != null && stats_heap.span_use[0].current > 0);
    Stats.heap_free(stats_heap, stats_memory);
    Stats.heap_release(stats_heap);
    Stats.finalize();
    check(Raw.rpmalloc_initialize() == 0);
    raw_heap := Raw.rpmalloc_allocator_proc(.CREATE_HEAP, 0, 0, null, null);

    check(raw_heap != null);
    raw_memory := Raw.rpmalloc_allocator_proc(.ALLOCATE, 4096, 0, null, raw_heap);

    check(raw_memory != null);
    (cast(*u8) raw_memory)[0] = 71;
    check(cast(u64) Raw.rpmalloc_allocator_proc(.IS_THIS_YOURS, 0, 0, raw_memory, raw_heap) == 1);
    Raw.rpmalloc_allocator_proc(.FREE, 0, 0, raw_memory, raw_heap);
    Raw.rpmalloc_allocator_proc(.DESTROY_HEAP, 0, 0, null, raw_heap);
    Raw.rpmalloc_allocator_proc(.SHUTDOWN, 0, 0, null, null);
}
