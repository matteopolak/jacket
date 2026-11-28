// Generated from docs/specs/stdlib/modules/rpmalloc.md.
// Regenerate with tools/native_bindings/generate.py; do not hand-edit.

#scope_file

#scope_export

// Types and layouts

// Options for the reallocation procedures, combined with `|`.
//
// - `NO_PRESERVE`: the old contents need not be copied to the new block.
// - `GROW_OR_FAIL`: the block is grown in place or the call returns null; it is not moved.
//
// Pass `0` for a plain resize that keeps the contents and may move the block.
Realloc_Flags :: enum_flags u32 {
    NO_PRESERVE :: 1;
    GROW_OR_FAIL :: 2;
}

#if !USE_RAW {
    // The settings [initialize_config] reads and [config] reports.
    //
    // - `memory_map`, `memory_unmap`: map and unmap the pages the allocator carves blocks from. When either is null, both are replaced by the built-in ones.
    // - `error_callback`: receives the allocator's error messages (and the source location of the report in the default build). When null, the built-in report is used.
    // - `map_fail_callback`: called with the size when mapping fails; its result is returned to the allocator. When null, the result is `0`.
    // - `page_size`, `span_size`, `span_map_count`: used only when the module is imported with `CONFIGURABLE := true`. `page_size` must be `0` or a power of two, and `span_size` `0` or a power of two from 4096 to 262144, or initialization fails.
    // - `enable_huge_pages`: asks the allocator to use huge pages.
    // - `page_name`, `huge_page_name`: names passed to the allocator for the mapped pages.
    //
    // After a successful initialization [config] holds the page, span and map-count values in use.
    rpmalloc_config_t :: struct {

        memory_map: #type (size: s64, offset: *s64) -> *void;

        memory_unmap: #type (address: *void, size: s64, offset: s64, release: s64) -> ();

        error_callback: #type (message: string, location: Source_Code_Location) -> ();

        map_fail_callback: #type (size: s64) -> s32;

        page_size: s64;

        span_size: s64;

        span_map_count: s64;

        enable_huge_pages: bool;

        page_name: string;
        huge_page_name: string;
    }
}

#if USE_RAW {
    // The settings [initialize_config] reads and [config] reports.
    //
    // - `memory_map`, `memory_unmap`: map and unmap the pages the allocator carves blocks from. When either is null, both are replaced by the built-in ones.
    // - `error_callback`: receives the allocator's error messages (and the source location of the report in the default build). When null, the built-in report is used.
    // - `map_fail_callback`: called with the size when mapping fails; its result is returned to the allocator. When null, the result is `0`.
    // - `page_size`, `span_size`, `span_map_count`: used only when the module is imported with `CONFIGURABLE := true`. `page_size` must be `0` or a power of two, and `span_size` `0` or a power of two from 4096 to 262144, or initialization fails.
    // - `enable_huge_pages`: asks the allocator to use huge pages.
    // - `page_name`, `huge_page_name`: names passed to the allocator for the mapped pages.
    //
    // After a successful initialization [config] holds the page, span and map-count values in use.
    rpmalloc_config_t :: struct {

        memory_map: #type (size: size_t, offset: *size_t) -> *void;

        memory_unmap: #type (address: *void, size: size_t, offset: size_t, release: size_t) -> ();

        error_callback: #type (message: string) -> ();

        map_fail_callback: #type (size: size_t) -> _int;

        page_size: size_t;

        span_size: size_t;

        span_map_count: size_t;

        enable_huge_pages: _int;

        page_name: string;
        huge_page_name: string;
    }
}

#if !USE_RAW {
    // The allocator-wide counters that [global_statistics] fills in: `mapped`, `mapped_peak`, `cached`, `huge_alloc`, `huge_alloc_peak`, `mapped_total` and `unmapped_total`.
    //
    // The counters are kept only when the module is imported with `ENABLE_STATISTICS := true`.
    rpmalloc_global_statistics_t :: struct {
        mapped: s64;
        mapped_peak: s64;
        cached: s64;
        huge_alloc: s64;
        huge_alloc_peak: s64;
        mapped_total: s64;
        unmapped_total: s64;
    }
}

#if USE_RAW {
    // The allocator-wide counters that [global_statistics] fills in: `mapped`, `mapped_peak`, `cached`, `huge_alloc`, `huge_alloc_peak`, `mapped_total` and `unmapped_total`.
    //
    // The counters are kept only when the module is imported with `ENABLE_STATISTICS := true`.
    rpmalloc_global_statistics_t :: struct {

        mapped: size_t;

        mapped_peak: size_t;

        cached: size_t;

        huge_alloc: size_t;

        huge_alloc_peak: size_t;

        mapped_total: size_t;

        unmapped_total: size_t;
    }
}

#if !USE_RAW {
    // The counters of the calling thread that [thread_statistics] fills in: the cache totals, `span_use` with one entry per span class and `size_use` with one entry per size class.
    //
    // The counters are kept only when the module is imported with `ENABLE_STATISTICS := true`.
    rpmalloc_thread_statistics_t :: struct {
        sizecache: s64;
        spancache: s64;
        thread_to_global: s64;
        global_to_thread: s64;

        span_use: [64] struct {
            current: s64;
            peak: s64;
            to_global: s64;
            from_global: s64;
            to_cache: s64;
            from_cache: s64;
            to_reserved: s64;
            from_reserved: s64;
            map_calls: s64;
        };

        size_use: [128] struct {
            alloc_current: s64;
            alloc_peak: s64;
            alloc_total: s64;
            free_total: s64;
            spans_to_cache: s64;
            spans_from_cache: s64;
            spans_from_reserved: s64;
            map_calls: s64;
        };
    }
}

#if USE_RAW {
    // The counters of the calling thread that [thread_statistics] fills in: the cache totals, `span_use` with one entry per span class and `size_use` with one entry per size class.
    //
    // The counters are kept only when the module is imported with `ENABLE_STATISTICS := true`.
    rpmalloc_thread_statistics_t :: struct {

        sizecache: size_t;

        spancache: size_t;

        thread_to_global: size_t;

        global_to_thread: size_t;

        span_use: [64] struct {

            current: size_t;

            peak: size_t;

            to_global: size_t;

            from_global: size_t;

            to_cache: size_t;

            from_cache: size_t;

            to_reserved: size_t;

            from_reserved: size_t;

            map_calls: size_t;
        };

        size_use: [128] struct {

            alloc_current: size_t;

            alloc_peak: size_t;

            alloc_total: size_t;

            free_total: size_t;

            spans_to_cache: size_t;

            spans_from_cache: size_t;

            spans_from_reserved: size_t;

            map_calls: size_t;
        };
    }
}

// Aliases and constants

#if USE_RAW {
    // Returns a pointer to the module's active configuration, not a copy.
    //
    // The values are those of the last successful [initialize_config], with the built-in mapping procedures filled in and the sizes the allocator chose. Before initialization and after [finalize] the configuration is zeroed.
    config :: rpmalloc_config;
}

#if USE_RAW {
    // Writes a report of the allocator's counters.
    //
    // With the default types, `builder` must be a `String_Builder`: the report is appended to it, and its `failed` flag is set when the counters cannot be captured. Without `ENABLE_STATISTICS` the call expands to nothing. With `USE_RAW := true` the argument is a C file pointer, and nothing is written unless the build has statistics and pretty-printing.
    dump_statistics :: rpmalloc_dump_statistics;
}

ENABLE_OVERRIDE :: false;

#if USE_RAW {
    // Shuts the allocator down. Does nothing when it is not initialized.
    //
    // Aborts the program when called while another allocator operation is running. After it returns, [is_initialized] is false and [config] holds a zeroed configuration.
    finalize :: rpmalloc_finalize;
}

#if USE_RAW {
    // Fills `stats` with the allocator-wide counters. They are kept only when the module is imported with `ENABLE_STATISTICS := true`.
    global_statistics :: rpmalloc_global_statistics;
}

#if USE_RAW {
    // The allocator's heap record. Heap procedures take and return `*Heap` handles; the fields are allocator state and are not meant to be written.
    Heap :: heap_t;
}

#if USE_RAW && (RPMALLOC_FIRST_CLASS_HEAPS) {
    // Returns a new heap, or null when the allocator is not initialized or no heap can be created.
    //
    // Free it with [heap_release]. A heap must not be used from two threads at once or from inside one of its own operations; either aborts the program.
    heap_acquire :: rpmalloc_heap_acquire;
}

#if USE_RAW && (RPMALLOC_FIRST_CLASS_HEAPS) {
    // Returns a block of at least `size` bytes aligned to `alignment` from `heap`, or null when `heap` is null, the request is invalid or the allocation fails.
    //
    // `alignment` follows the rules of [rpaligned_alloc].
    heap_aligned_alloc :: rpmalloc_heap_aligned_alloc;
}

#if USE_RAW && (RPMALLOC_FIRST_CLASS_HEAPS) {
    // Resizes the block at `ptr` to `size` bytes aligned to `alignment` in `heap` and returns the resulting block, or null when the request is invalid or fails.
    //
    // `alignment` follows the rules of [rpaligned_alloc]; `flags` selects the behaviour of [Realloc_Flags].
    heap_aligned_realloc :: rpmalloc_heap_aligned_realloc;
}

#if USE_RAW && (RPMALLOC_FIRST_CLASS_HEAPS) {
    // Returns a block of at least `size` bytes from `heap`, or null when `heap` is null, `size` is negative or too large, or the allocation fails.
    //
    // A handle that is invalid or released aborts the program.
    heap_alloc :: rpmalloc_heap_alloc;
}

#if USE_RAW && (RPMALLOC_FIRST_CLASS_HEAPS) {
    // Frees the block at `ptr`. `heap` is only checked to be a live handle (or null): the block returns to the heap it came from. Does nothing when `ptr` is null.
    heap_free :: rpmalloc_heap_free;
}

#if USE_RAW && (RPMALLOC_FIRST_CLASS_HEAPS) {
    // Frees every block allocated from `heap`.
    heap_free_all :: rpmalloc_heap_free_all;
}

#if USE_RAW && (RPMALLOC_FIRST_CLASS_HEAPS) {
    // Resizes the block at `ptr` to `size` bytes in `heap` and returns the resulting block, or null when the request is invalid or fails.
    //
    // `flags` selects the behaviour of [Realloc_Flags].
    heap_realloc :: rpmalloc_heap_realloc;
}

#if USE_RAW && (RPMALLOC_FIRST_CLASS_HEAPS) {
    // Releases `heap`, which [heap_acquire] returned. The handle must not be used afterwards.
    //
    // A null `heap` is ignored. A handle that is invalid or already released aborts the program.
    heap_release :: rpmalloc_heap_release;
}

#if USE_RAW && (RPMALLOC_FIRST_CLASS_HEAPS) {
    // Makes `heap` the calling thread's heap and releases the thread's previous heap when it differs. Treat the previous handle as released.
    //
    // A null `heap` releases the thread's current heap and leaves the thread without one.
    heap_thread_set_current :: rpmalloc_heap_thread_set_current;
}

#if USE_RAW {
    // Initializes the allocator and the calling thread, and returns `0` on success or a nonzero error code.
    //
    // Call it before any allocation. When the allocator is already initialized it only initializes the calling thread and returns `0`. Undo it with [finalize].
    initialize :: rpmalloc_initialize;
}

#if USE_RAW {
    // Initializes the allocator with `config` and returns `0` on success or a nonzero error code.
    //
    // A null `config` behaves like [initialize]. `page_size`, `span_size` and `span_map_count` are honoured only when `RPMALLOC_CONFIGURABLE` is true; a `page_size` or `span_size` that is not allowed returns `22`. The settings are copied; afterwards [config] reports the values in use. When the allocator is already initialized only the calling thread is initialized.
    initialize_config :: rpmalloc_initialize_config;
}

#if USE_RAW {
    // Returns whether [initialize] or [initialize_config] has succeeded and [finalize] has not run since.
    is_initialized :: rpmalloc_is_initialized;
}

#if USE_RAW {
    // Returns whether the calling thread has been initialized.
    is_thread_initialized :: rpmalloc_is_thread_initialized;
}

// Whether the module was imported with `CONFIGURABLE := true`, which is when [initialize_config] applies `page_size`, `span_size` and `span_map_count`.
RPMALLOC_CONFIGURABLE :: CONFIGURABLE;

#if USE_RAW && (RPMALLOC_FIRST_CLASS_HEAPS) {
    // The heap record that the `rpmalloc_heap_*` procedures take and return a pointer to. The fields are allocator state and are not meant to be written.
    rpmalloc_heap_t :: heap_t;
}

#if USE_RAW {
    // Marks the calling thread as the main thread.
    set_main_thread :: rpmalloc_set_main_thread;
}

#if USE_RAW {
    // Finishes the calling thread's use of the allocator. `release_caches` is passed through to the native allocator.
    thread_finalize :: rpmalloc_thread_finalize;
}

#if USE_RAW {
    // Prepares the calling thread to allocate.
    thread_initialize :: rpmalloc_thread_initialize;
}

#if USE_RAW {
    // Fills `stats` with the counters of the calling thread. They are kept only when the module is imported with `ENABLE_STATISTICS := true`.
    thread_statistics :: rpmalloc_thread_statistics;
}

#if USE_RAW {
    // Returns the usable size in bytes of the block at `ptr`.
    usable_size :: rpmalloc_usable_size;
}

// Native storage

errno: enum s32 {
    SUCCESS :: 0;
    ENOMEM :: 12;
    EINVAL :: 22;
}
