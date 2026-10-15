// Private ABI evidence from reviewed raw records and independent pinned SDK
// atomic probes. Public pointers are compared with, never cast into, SDK heaps.
#import "Basic";
Raw :: #import "rpmalloc"(USE_RAW = true);
Small :: #import "rpmalloc"(USE_RAW = true, ENABLE_STATISTICS = false);
RawBridge :: #import "internal/bindings/rpmalloc-heaps"(RAW = true);
RawNoStatsBridge :: #import "internal/bindings/rpmalloc-heaps"(STATISTICS = false, RAW = true);
Native :: #import "internal/bindings/rpmalloc-sdk"(STATISTICS = true, RAW = true, PUBLIC_HEAPS = true);

probe :: #library,system "rpmalloc_storage_probe";
native_counter :: (heap: *void, family: u32, index: u32, field: u32) -> s64 #foreign probe "binding_rpmalloc_native_heap_counter";
public_atomic32 :: (address: *void) -> s64 #foreign probe "binding_rpmalloc_public_atomic32";

#load "generated/rpmalloc-raw-heap-facade-layout.jai";

span_counter :: (heap: *$T, index: s64, field: s64) -> s64 {
    row := *heap.span_use[index];
    if field == 0 return public_atomic32(cast(*void) *row.current);
    if field == 1 return row.high;
    if field == 2 return public_atomic32(cast(*void) *row.spans_deferred);
    if field == 3 return public_atomic32(cast(*void) *row.spans_to_global);
    if field == 4 return public_atomic32(cast(*void) *row.spans_from_global);
    if field == 5 return public_atomic32(cast(*void) *row.spans_to_cache);
    if field == 6 return public_atomic32(cast(*void) *row.spans_from_cache);
    if field == 7 return public_atomic32(cast(*void) *row.spans_to_reserved);
    if field == 8 return public_atomic32(cast(*void) *row.spans_from_reserved);

    return public_atomic32(cast(*void) *row.spans_map_calls);
}

size_counter :: (heap: *$T, index: s64, field: s64) -> s64 {
    row := *heap.size_class_use[index];
    if field == 0 return public_atomic32(cast(*void) *row.alloc_current);
    if field == 1 return row.alloc_peak;
    if field == 2 return public_atomic32(cast(*void) *row.alloc_total);
    if field == 3 return public_atomic32(cast(*void) *row.free_total);
    if field == 4 return public_atomic32(cast(*void) *row.spans_current);
    if field == 5 return row.spans_peak;
    if field == 6 return public_atomic32(cast(*void) *row.spans_to_cache);
    if field == 7 return public_atomic32(cast(*void) *row.spans_from_cache);
    if field == 8 return public_atomic32(cast(*void) *row.spans_from_reserved);
    if field == 9 return public_atomic32(cast(*void) *row.spans_map_calls);

    return public_atomic32(cast(*void) *row.unused);
}

main :: () {
    assert(Raw.initialize() == 0 && Small.initialize() == 0);
    heap := Raw.heap_acquire();
    other := Small.heap_acquire();

    assert(heap != null && other != null);
    canonical_layout_raw_statistics(heap);
    canonical_layout_raw_no_statistics(other);
    memory := Raw.heap_alloc(heap, 512);
    second := Raw.heap_alloc(heap, 32768);

    assert(memory != null && second != null);
    native := RawBridge.native_heap(cast(*void) heap);

    #assert type_of(native) == *Native.rpmalloc_heap_t;
    assert(native != null && cast(*void) native != cast(*void) heap);

    for i: 0..62 {
        for j: 0..9 assert(span_counter(heap, i, j) == native_counter(cast(*void) native, 0, cast(u32) i, cast(u32) j));
    }
    for i: 0..126 {
        for j: 0..10 assert(size_counter(heap, i, j) == native_counter(cast(*void) native, 1, cast(u32) i, cast(u32) j));
    }

    Raw.heap_free(heap, memory);
    Raw.heap_free(heap, second);
    Raw.heap_release(heap);
    Small.heap_release(other);
    Small.finalize();
    Raw.finalize();
}
