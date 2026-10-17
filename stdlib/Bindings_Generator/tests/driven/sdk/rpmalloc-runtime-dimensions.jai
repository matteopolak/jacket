// Reviewed positive numeric policies; no assumptions about inactive metadata.
Small :: #import "rpmalloc"(HEAP_ARRAY_SIZE = 17, DEFAULT_SPAN_MAP_COUNT = 3, GLOBAL_CACHE_MULTIPLIER = 2);
Large :: #import "rpmalloc"(HEAP_ARRAY_SIZE = 53, DEFAULT_SPAN_MAP_COUNT = 96, GLOBAL_CACHE_MULTIPLIER = 3);
TooLarge :: #import "rpmalloc"(HEAP_ARRAY_SIZE = 0x7fff_ffff_ffff_ffff);
TooManySpans :: #import "rpmalloc"(DEFAULT_SPAN_MAP_COUNT = 0x1_0000_0000);

libc :: #library,system "c";
abort :: () #foreign libc;

check :: (condition: bool) {
    if !condition abort();
}

exercise :: (proc: Allocator_Proc) {
    heap := proc(.CREATE_HEAP, 0, 0, null, null);

    check(heap != null);
    memory := proc(.ALLOCATE, 8192, 0, null, heap);

    check(memory != null);
    (cast(*u8) memory)[0] = 71;
    resized := proc(.RESIZE, 131072, 0, memory, heap);

    check(resized != null && (cast(*u8) resized)[0] == 71);
    check(cast(u64) proc(.IS_THIS_YOURS, 0, 0, resized, heap) == 1);
    proc(.FREE, 0, 0, resized, heap);
    proc(.DESTROY_HEAP, 0, 0, null, heap);
    proc(.SHUTDOWN, 0, 0, null, null);
}

main :: () {
    // The very first native-domain attempt may fail before any table exists.
    check(TooLarge.initialize() == 12);
    check(!TooLarge.is_initialized());
    check(Small.initialize() == 0);
    check(Small.config().span_map_count == 3);
    // Distinct argument lists own independent domains even with one SDK image.
    check(Large.initialize() == 0);
    check(Large.config().span_map_count == 96);
    live_large := Large.rpmalloc(4096);

    check(live_large != null);
    (cast(*u8) live_large)[0] = 74;
    check(Small.config().span_map_count == 3);
    exercise(Small.rpmalloc_allocator_proc);
    check(Large.is_initialized() && (cast(*u8) live_large)[0] == 74);
    Large.rpfree(live_large);
    Large.finalize();
    check(TooLarge.initialize() == 12);
    check(!TooLarge.is_initialized());
    check(TooManySpans.initialize() == 12);
    check(!TooManySpans.is_initialized());
    check(Large.initialize() == 0);
    check(Large.config().span_map_count == 96);
    exercise(Large.rpmalloc_allocator_proc);
    check(Small.initialize() == 0);
    check(Small.config().span_map_count == 3);
    exercise(Small.rpmalloc_allocator_proc);
}
