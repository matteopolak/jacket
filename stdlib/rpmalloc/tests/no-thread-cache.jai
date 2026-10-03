// Reviewed rpmalloc.md conditional Heap fields and generic allocator protocol.
// Private test boundary uses actual independently generated C mirror offsets;
// public heap pointers are never passed as native SDK heap pointers.
#import "Basic";
#load "../../../tests/native-bindings/rpmalloc-cache-exercise.jai";
Settings :: #import "internal/bindings/rpmalloc-build";
RPM :: #import "rpmalloc"(ENABLE_THREAD_CACHE = false, ENABLE_GLOBAL_CACHE = false);
Small :: #import "rpmalloc"(ENABLE_THREAD_CACHE = false, ENABLE_GLOBAL_CACHE = false, ENABLE_STATISTICS = false);
BothOff :: #import "rpmalloc"(ENABLE_THREAD_CACHE = false, ENABLE_GLOBAL_CACHE = false);
BothOffSmall :: #import "rpmalloc"(ENABLE_THREAD_CACHE = false, ENABLE_GLOBAL_CACHE = false, ENABLE_STATISTICS = false);
Raw :: #import "rpmalloc"(USE_RAW = true, ENABLE_THREAD_CACHE = false, ENABLE_GLOBAL_CACHE = false);
RawSmall :: #import "rpmalloc"(USE_RAW = true, ENABLE_THREAD_CACHE = false, ENABLE_GLOBAL_CACHE = false, ENABLE_STATISTICS = false);
RawBothOff :: #import "rpmalloc"(USE_RAW = true, ENABLE_THREAD_CACHE = false, ENABLE_GLOBAL_CACHE = false);
RawBothOffSmall :: #import "rpmalloc"(USE_RAW = true, ENABLE_THREAD_CACHE = false, ENABLE_GLOBAL_CACHE = false, ENABLE_STATISTICS = false);
Bridge :: #import "internal/bindings/rpmalloc-heaps"(THREAD_CACHE = false, GLOBAL_CACHE = false);
Domain :: #import "internal/bindings/rpmalloc-domain"(Owner = RPM.rpmalloc_config_t, STATISTICS = true, RAW = false, PUBLIC_HEAPS = true, GLOBAL_CACHE = false, THREAD_CACHE = false, ASSERTS = true, VALIDATE_ARGS = true, ADAPTIVE = false);
NoStatsBridge :: #import "internal/bindings/rpmalloc-heaps"(STATISTICS = false, THREAD_CACHE = false, GLOBAL_CACHE = false);
RawBridge :: #import "internal/bindings/rpmalloc-heaps"(RAW = true, PUBLIC_HEAPS = Settings.FIRST_CLASS_HEAPS, THREAD_CACHE = false, GLOBAL_CACHE = false);
RawNoStatsBridge :: #import "internal/bindings/rpmalloc-heaps"(STATISTICS = false, RAW = true, PUBLIC_HEAPS = Settings.FIRST_CLASS_HEAPS, THREAD_CACHE = false, GLOBAL_CACHE = false);

RawBaseBridge :: RawBridge;
RawBaseNoStatsBridge :: RawNoStatsBridge;

#load "../../../tests/native-bindings/generated/rpmalloc-no-thread-cache-rewrite-layout.jai";

#if Settings.FIRST_CLASS_HEAPS {
    #load "../../../tests/native-bindings/generated/rpmalloc-no-thread-cache-raw-layout.jai";
} else {
    #load "../../../tests/native-bindings/generated/rpmalloc-no-thread-cache-raw-base-layout.jai";
}

main :: () {
    // Repeated imports with identical parameters share canonical module state.
    assert(cast(*void) RPM.config() == cast(*void) BothOff.config());
    assert(cast(*void) Raw.config() == cast(*void) RawBothOff.config());
    cache_exercise(RPM.rpmalloc_allocator_proc);
    cache_exercise(Small.rpmalloc_allocator_proc);
    cache_exercise(BothOff.rpmalloc_allocator_proc);
    cache_exercise(BothOffSmall.rpmalloc_allocator_proc);
    cache_exercise(Raw.rpmalloc_allocator_proc);
    cache_exercise(RawSmall.rpmalloc_allocator_proc);
    cache_exercise(RawBothOff.rpmalloc_allocator_proc);
    cache_exercise(RawBothOffSmall.rpmalloc_allocator_proc);
    heap := RPM.heap_acquire();
    small := Small.heap_acquire();
    raw := cast(*Raw.Heap) Raw.rpmalloc_allocator_proc(.CREATE_HEAP, 0, 0, null, null);
    raw_small := cast(*RawSmall.Heap) RawSmall.rpmalloc_allocator_proc(.CREATE_HEAP, 0, 0, null, null);

    assert(heap != null && small != null && raw != null && raw_small != null);
    canonical_layout_no_thread_cache_statistics(heap);
    canonical_layout_no_thread_cache_no_statistics(small);

    #if Settings.FIRST_CLASS_HEAPS {
        canonical_layout_no_thread_cache_raw_statistics(raw);
        canonical_layout_no_thread_cache_raw_no_statistics(raw_small);
    } else {
        canonical_layout_no_thread_cache_raw_base_statistics(raw);
        canonical_layout_no_thread_cache_raw_base_no_statistics(raw_small);
    }

    previous := Domain.select();
    native := Bridge.native_heap(heap);

    Domain.restore(previous);
    assert(native != cast(*void) heap);
    RPM.heap_release(heap);
    Small.heap_release(small);
    Raw.rpmalloc_allocator_proc(.DESTROY_HEAP, 0, 0, null, raw);
    RawSmall.rpmalloc_allocator_proc(.DESTROY_HEAP, 0, 0, null, raw_small);
    RPM.finalize();
    Small.finalize();
    BothOff.finalize();
    BothOffSmall.finalize();
    Raw.finalize();
    RawSmall.finalize();
    RawBothOff.finalize();
    RawBothOffSmall.finalize();
}
