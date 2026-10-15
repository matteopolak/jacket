// Reviewed raw Heap alias remains a canonical complete record with the named
// first-class interface disabled. Generic CREATE_HEAP returns that shape.
#import "Basic";
Raw :: #import "rpmalloc"(USE_RAW = true);
Small :: #import "rpmalloc"(USE_RAW = true, ENABLE_STATISTICS = false);
RawBaseBridge :: #import "internal/bindings/rpmalloc-heaps"(RAW = true, PUBLIC_HEAPS = false, GLOBAL_CACHE = true, THREAD_CACHE = true);
RawBaseNoStatsBridge :: #import "internal/bindings/rpmalloc-heaps"(STATISTICS = false, RAW = true, PUBLIC_HEAPS = false, GLOBAL_CACHE = true, THREAD_CACHE = true);
Native :: #import "internal/bindings/rpmalloc-sdk"(STATISTICS = true, RAW = true, PUBLIC_HEAPS = false, GLOBAL_CACHE = true, THREAD_CACHE = true);

#load "generated/rpmalloc-raw-base-heap-facade-layout.jai";

main :: () {
    assert(Raw.initialize() == 0 && Small.initialize() == 0);
    handle := Raw.rpmalloc_allocator_proc(.CREATE_HEAP, 0, 0, null, null);
    other_handle := Small.rpmalloc_allocator_proc(.CREATE_HEAP, 0, 0, null, null);

    assert(handle != null && other_handle != null);
    heap := cast(*Raw.Heap) handle;
    other := cast(*Small.Heap) other_handle;

    #assert type_info(Raw.Heap).name == "heap_t";
    canonical_layout_raw_base_statistics(heap);
    canonical_layout_raw_base_no_statistics(other);
    native := RawBaseBridge.native_heap(handle);

    #assert type_of(native) == *Native.rpmalloc_heap_t;
    assert(cast(*void) native != handle);
    memory := Raw.rpmalloc_allocator_proc(.ALLOCATE, 301, 0, null, handle);

    assert(memory != null);
    assert(cast(u64) Raw.rpmalloc_allocator_proc(.IS_THIS_YOURS, 0, 0, memory, handle) == 1);
    Raw.rpmalloc_allocator_proc(.FREE, 0, 0, memory, handle);
    Raw.rpmalloc_allocator_proc(.DESTROY_HEAP, 0, 0, null, handle);
    Small.rpmalloc_allocator_proc(.DESTROY_HEAP, 0, 0, null, other_handle);
    Small.finalize();
    Raw.finalize();
}
