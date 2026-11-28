// A general-purpose allocator for programs that allocate from many threads, with per-thread caches and separate heaps.
//
// Call [initialize] (or [initialize_config]) before allocating and [finalize] at the end. Allocate with [rpmalloc],
// [rpaligned_alloc] and [rpcalloc]'s relatives, resize with [rprealloc], and free with [rpfree]. [rpmalloc_allocator_proc]
// is the `Allocator` procedure that routes the default allocation calls here. [heap_acquire] gives a separate heap that
// [heap_free_all] empties in one call.
//
// Every import with the same module parameters shares one allocator, and imports with different parameters get separate
// ones. The parameters turn on statistics, the caches, argument checks and the settings [initialize_config] accepts
// (`CONFIGURABLE`); with `USE_RAW := true` the procedures take the C types and keep their C names.

#module_parameters(
    // The size of the table of heaps that threads share.
    HEAP_ARRAY_SIZE := 47,
    // How many spans are mapped at a time.
    DEFAULT_SPAN_MAP_COUNT := 64,
    // How many times larger the global cache is than a thread cache.
    GLOBAL_CACHE_MULTIPLIER := 8,
    // Keeps freed spans in a cache for each thread.
    ENABLE_THREAD_CACHE := true,
    // Keeps freed spans in a cache that threads share. Needs the thread cache.
    ENABLE_GLOBAL_CACHE := true,
    // Counts allocations and mapped memory for the statistics procedures.
    ENABLE_STATISTICS := true,
    // Checks the allocator's internal invariants.
    ENABLE_ASSERTS := true,
    // Checks the arguments of its procedures.
    ENABLE_VALIDATE_ARGS := true,
    // The advice given to Linux when pages are freed.
    LINUX_FREE_ADVICE: enum {
        DONTNEED;
        FREE;
    } = .DONTNEED,
    // Lets the allocator be configured when it is initialized.
    CONFIGURABLE := false,
    // Sizes each thread's cache from how the thread allocates. Needs the thread cache.
    ENABLE_ADAPTIVE_THREAD_CACHE := false,
    // Asks Linux for transparent huge pages.
    LINUX_HUGE_PAGES := false,
    // Binds the C interface with `size_t` and the C types as the header writes them, not `s64`.
    USE_RAW := false,
    // Never returns memory to the OS. Needs the global and unlimited caches.
    DISABLE_UNMAP := false,
    // Lets the caches grow without a limit. Needs the global cache.
    ENABLE_UNLIMITED_CACHE := false
)();

#assert HEAP_ARRAY_SIZE > 0, "HEAP_ARRAY_SIZE must be positive";
#assert DEFAULT_SPAN_MAP_COUNT > 0, "DEFAULT_SPAN_MAP_COUNT must be positive";
#assert GLOBAL_CACHE_MULTIPLIER > 0, "GLOBAL_CACHE_MULTIPLIER must be positive";
#assert !ENABLE_GLOBAL_CACHE || ENABLE_THREAD_CACHE, "ENABLE_GLOBAL_CACHE requires ENABLE_THREAD_CACHE";
#assert !ENABLE_UNLIMITED_CACHE || ENABLE_GLOBAL_CACHE, "ENABLE_UNLIMITED_CACHE requires ENABLE_GLOBAL_CACHE";
#assert !DISABLE_UNMAP || (ENABLE_GLOBAL_CACHE && ENABLE_UNLIMITED_CACHE), "DISABLE_UNMAP requires global and unlimited caches";
#assert !ENABLE_ADAPTIVE_THREAD_CACHE || ENABLE_THREAD_CACHE, "ENABLE_ADAPTIVE_THREAD_CACHE requires ENABLE_THREAD_CACHE";

#load "context.jai";
#load "declarations.jai";
#load "wrappers.jai";
#load "allocator.jai";
#load "statistics.jai";
