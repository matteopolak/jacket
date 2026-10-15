// ABI oracle: complete pinned official rpmalloc C source, with SDK build flags.
// Public type oracle: reviewed rpmalloc.md contextual shapes. This private unit
// proves storage separation; it is not a public heap implementation.
#import "Basic";
Native :: #import "internal/bindings/rpmalloc-sdk";
Storage :: #import "internal/bindings/rpmalloc-storage";

probe :: #library,system "rpmalloc_storage_probe";
storage_size :: (index: u32) -> u64 #foreign probe "binding_rpmalloc_storage_size";
storage_offset :: (index: u32) -> u64 #foreign probe "binding_rpmalloc_storage_offset";
storage_field_size :: (index: u32) -> u64 #foreign probe "binding_rpmalloc_storage_field_size";
native_id :: (heap: *void) -> s64 #foreign probe "binding_rpmalloc_native_heap_id";
native_owner :: (heap: *void) -> u64 #foreign probe "binding_rpmalloc_native_heap_owner";

USE_RAW :: false;
ENABLE_THREAD_CACHE :: true;
ENABLE_ADAPTIVE_THREAD_CACHE :: false;
ENABLE_STATISTICS :: true;

#load "../../stdlib/rpmalloc/context.jai";
#load "generated/rpmalloc-storage-layout.jai";

#scope_export
main :: () {
    storage_layout_checks();
    public_storage_separation_checks();
    assert(Native.rpmalloc_initialize() == 0);
    heap := Native.rpmalloc_heap_acquire();

    assert(heap != null);
    // Valid only while this SDK heap is alive and this thread owns its use.
    // Atomic members are intentionally not read through plain storage views.
    view := cast(*Storage.Heap_Storage) heap;

    assert(cast(s64) view.id == native_id(cast(*void) heap));
    assert(view.owner_thread == native_owner(cast(*void) heap));
    memory := Native.rpmalloc_heap_alloc(heap, 301);

    assert(memory != null);
    assert(Native.rpmalloc_get_heap_for_ptr(memory) == heap);
    Native.rpmalloc_heap_free(heap, memory);
    Native.rpmalloc_heap_release(heap);
    Native.rpmalloc_finalize();
}
