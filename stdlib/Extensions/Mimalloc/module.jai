// mimalloc as a `context.allocator`.
//
// Binds the mimalloc C library (pinned in `tools/native/native-libs.json`, built into `target/native-libs`) on macOS
// and Linux. `mimalloc_allocator` allocates from the calling thread's heap; `make_heap` creates a separate heap that
// `release_heap` frees in one call.
//
// ```jai
// #import "Extensions/Mimalloc";
//
// context.allocator = mimalloc_allocator;
// numbers := NewArray(100, int);
// assert(numbers.count == 100);
// array_free(numbers);
// ```

// test: only(.LINUX,.MACOS)

#if OS == .MACOS || OS == .LINUX {
    CAPS_VERSION_STRING :: "Extensions/Mimalloc";

    // Allocates from the calling thread's own heap and may be used from every thread. Memory can be freed from any thread.
    //
    // ```jai
    // context.allocator = mimalloc_allocator;
    // block := alloc(64);
    // assert(usable_size(block) >= 64);
    // free(block);
    // ```
    mimalloc_allocator :: Allocator.{ mimalloc_proc, null };

    // Creates a heap whose allocations `release_heap` frees together. Allocate from a heap on the thread that made it;
    // freeing its blocks works from any thread, and `release_heap` must run on a thread that has stopped using it.
    //
    // ```jai
    // heap := make_heap();
    // block := alloc(32,, heap);
    // assert(block != null);
    // release_heap(heap); // frees `block` too
    // ```
    make_heap :: () -> Allocator {
        return .{ mimalloc_proc, mi_heap_new() };
    }

    // Frees every block of a heap made by `make_heap`. Pointers from it are invalid afterwards.
    //
    // ```jai
    // heap := make_heap();
    // alloc(32,, heap);
    // alloc(64,, heap);
    // release_heap(heap);
    // ```
    release_heap :: (heap: Allocator) {
        assert(heap.proc == mimalloc_proc && heap.data != null, "release_heap needs an allocator from make_heap");
        mi_heap_destroy(heap.data);
    }

    // The number of bytes a block from this allocator can hold, at least what was requested.
    //
    // ```jai
    // context.allocator = mimalloc_allocator;
    // block := alloc(10);
    // assert(usable_size(block) >= 10);
    // free(block);
    // ```
    usable_size :: (block: *void) -> s64 {
        return cast(s64) mi_usable_size(block);
    }

    // Allocates `size` bytes aligned to `alignment`, a power of two. Free the block with `free` through the same allocator.
    //
    // ```jai
    // block := allocate_aligned(100, 4096);
    // assert((cast(u64) block & 4095) == 0);
    // mimalloc_proc(.FREE, 0, 0, block, null);
    // ```
    allocate_aligned :: (size: s64, alignment: s64, heap := mimalloc_allocator) -> *void {
        assert(size >= 0 && alignment > 0 && (alignment & (alignment - 1)) == 0, "invalid mimalloc aligned request");
        if heap.data return mi_heap_malloc_aligned(heap.data, cast(u64) size, cast(u64) alignment);
        return mi_malloc_aligned(cast(u64) size, cast(u64) alignment);
    }

    // Returns unused memory to the operating system now instead of when mimalloc decides to.
    //
    // ```jai
    // collect();
    // ```
    collect :: (force := false) {
        mi_collect(force);
    }

    // The `Allocator_Proc` of mimalloc: use it as `Allocator.{ mimalloc_proc, null }` or with a heap from `make_heap`.
    // Resize keeps the block in place when it fits.
    //
    // ```jai
    // block := mimalloc_proc(.ALLOCATE, 16, 0, null, null);
    // block = mimalloc_proc(.RESIZE, 64, 16, block, null);
    // mimalloc_proc(.FREE, 0, 0, block, null);
    // ```
    mimalloc_proc :: (mode: Allocator_Mode, requested_size: s64, old_size: s64, old_memory: *void, allocator_data: *void) -> *void {
        if mode == .ALLOCATE {
            if allocator_data return mi_heap_malloc(allocator_data, cast(u64) requested_size);
            return mi_malloc(cast(u64) requested_size);
        }

        if mode == .RESIZE {
            if allocator_data return mi_heap_realloc(allocator_data, old_memory, cast(u64) requested_size);
            return mi_realloc(old_memory, cast(u64) requested_size);
        }

        if mode == .FREE {
            mi_free(old_memory);
            return null;
        }

        if mode == .IS_THIS_YOURS {
            if old_memory && mi_is_in_heap_region(old_memory) return cast(*void) 1;
            return null;
        }

        if mode == .CAPS {
            if old_memory {
                (cast(*string) old_memory).* = CAPS_VERSION_STRING;
            }

            return cast(*void) cast(u64) (Allocator_Caps.MULTIPLE_THREADS | .CREATE_HEAP | .FREE | .ACTUALLY_RESIZE | .IS_THIS_YOURS | .HINT_I_AM_A_GENERAL_HEAP_ALLOCATOR);
        }

        if mode == .CREATE_HEAP return mi_heap_new();

        if mode == .DESTROY_HEAP {
            if allocator_data mi_heap_destroy(allocator_data);
            return null;
        }

        if mode == .THREAD_STOP {
            mi_thread_done();
            return null;
        }

        return null; // STARTUP, SHUTDOWN and THREAD_START need nothing.
    }

    #scope_file

    #import "Basic";

    libmimalloc :: #library,system "mimalloc";

    mi_malloc :: (size: u64) -> *void #foreign libmimalloc;
    mi_realloc :: (block: *void, size: u64) -> *void #foreign libmimalloc;
    mi_free :: (block: *void) #foreign libmimalloc;
    mi_usable_size :: (block: *void) -> u64 #foreign libmimalloc;
    mi_malloc_aligned :: (size: u64, alignment: u64) -> *void #foreign libmimalloc;
    mi_is_in_heap_region :: (block: *void) -> bool #foreign libmimalloc;
    mi_collect :: (force: bool) #foreign libmimalloc;
    mi_thread_done :: () #foreign libmimalloc;
    mi_heap_new :: () -> *void #foreign libmimalloc;
    mi_heap_destroy :: (heap: *void) #foreign libmimalloc;
    mi_heap_malloc :: (heap: *void, size: u64) -> *void #foreign libmimalloc;
    mi_heap_realloc :: (heap: *void, block: *void, size: u64) -> *void #foreign libmimalloc;
    mi_heap_malloc_aligned :: (heap: *void, size: u64, alignment: u64) -> *void #foreign libmimalloc;
} else {
    #assert false, "Extensions/Mimalloc supports macOS and Linux only; use Extensions/Tlsf or the default allocator elsewhere";
}
