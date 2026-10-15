// Private integration evidence: reviewed public record/C mirror layouts and
// independently compiled pinned SDK storage/atomic counter probes.
#import "Basic";
RPM :: #import "rpmalloc";
Small :: #import "rpmalloc"(ENABLE_STATISTICS = false);
Bridge :: #import "internal/bindings/rpmalloc-heaps";
NoStatsBridge :: #import "internal/bindings/rpmalloc-heaps"(STATISTICS = false);
Native :: #import "internal/bindings/rpmalloc-sdk"(STATISTICS = true, RAW = false, PUBLIC_HEAPS = true);

probe :: #library,system "rpmalloc_storage_probe";
native_id :: (heap: *void) -> s64 #foreign probe "binding_rpmalloc_native_heap_id";
native_owner :: (heap: *void) -> u64 #foreign probe "binding_rpmalloc_native_heap_owner";
native_counter :: (heap: *void, family: u32, index: u32, field: u32) -> s64 #foreign probe "binding_rpmalloc_native_heap_counter";

#load "generated/rpmalloc-heap-facade-layout.jai";

span_counter :: (heap: *$T, index: s64, field: s64) -> s64 {
    row := *heap.span_use[index];
    if field == 0 return row.current;
    if field == 1 return row.high;
    if field == 2 return row.spans_deferred;
    if field == 3 return row.spans_to_global;
    if field == 4 return row.spans_from_global;
    if field == 5 return row.spans_to_cache;
    if field == 6 return row.spans_from_cache;
    if field == 7 return row.spans_to_reserved;
    if field == 8 return row.spans_from_reserved;

    return row.spans_map_calls;
}

size_counter :: (heap: *$T, index: s64, field: s64) -> s64 {
    row := *heap.size_class_use[index];
    if field == 0 return row.alloc_current;
    if field == 1 return row.alloc_peak;
    if field == 2 return row.alloc_total;
    if field == 3 return row.free_total;
    if field == 4 return row.spans_current;
    if field == 5 return row.spans_peak;
    if field == 6 return row.spans_to_cache;
    if field == 7 return row.spans_from_cache;
    if field == 8 return row.spans_from_reserved;
    if field == 9 return row.spans_map_calls;

    return row.unused;
}

main :: () {
    assert(RPM.initialize() == 0 && Small.initialize() == 0);
    heap := RPM.heap_acquire();
    other := Small.heap_acquire();

    assert(heap != null && other != null);
    canonical_layout_statistics(heap);
    canonical_layout_no_statistics(other);
    memory := RPM.heap_alloc(heap, 512);
    second := RPM.heap_alloc(heap, 32768);

    assert(memory != null && second != null);
    native := Bridge.native_heap(cast(*void) heap);

    #assert type_of(native) == *Native.rpmalloc_heap_t;
    assert(native != null && cast(*void) native != cast(*void) heap);
    assert(cast(s64) heap.id == native_id(cast(*void) native));
    assert(heap.owner_thread == native_owner(cast(*void) native));

    for i: 0..62 {
        for j: 0..9 assert(span_counter(heap, i, j) == native_counter(cast(*void) native, 0, cast(u32) i, cast(u32) j));
    }
    for i: 0..126 {
        for j: 0..10 assert(size_counter(heap, i, j) == native_counter(cast(*void) native, 1, cast(u32) i, cast(u32) j));
    }

    RPM.heap_free(heap, memory);
    RPM.heap_free(heap, second);
    RPM.heap_release(heap);
    Small.heap_release(other);
    Small.finalize();
    RPM.finalize();
}
