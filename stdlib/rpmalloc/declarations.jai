// Generated from docs/specs/stdlib/modules/rpmalloc.md.
// Regenerate with tools/native_bindings/generate.py; do not hand-edit.

#scope_file

#scope_export

// Types and layouts

Realloc_Flags :: enum_flags u32 {
    NO_PRESERVE :: 1;
    GROW_OR_FAIL :: 2;
}

#if !USE_RAW {
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
    config :: rpmalloc_config;
}

#if USE_RAW {
    dump_statistics :: rpmalloc_dump_statistics;
}

ENABLE_OVERRIDE :: false;

#if USE_RAW {
    finalize :: rpmalloc_finalize;
}

#if USE_RAW {
    global_statistics :: rpmalloc_global_statistics;
}

#if USE_RAW {
    Heap :: heap_t;
}

#if USE_RAW && (RPMALLOC_FIRST_CLASS_HEAPS) {
    heap_acquire :: rpmalloc_heap_acquire;
}

#if USE_RAW && (RPMALLOC_FIRST_CLASS_HEAPS) {
    heap_aligned_alloc :: rpmalloc_heap_aligned_alloc;
}

#if USE_RAW && (RPMALLOC_FIRST_CLASS_HEAPS) {
    heap_aligned_realloc :: rpmalloc_heap_aligned_realloc;
}

#if USE_RAW && (RPMALLOC_FIRST_CLASS_HEAPS) {
    heap_alloc :: rpmalloc_heap_alloc;
}

#if USE_RAW && (RPMALLOC_FIRST_CLASS_HEAPS) {
    heap_free :: rpmalloc_heap_free;
}

#if USE_RAW && (RPMALLOC_FIRST_CLASS_HEAPS) {
    heap_free_all :: rpmalloc_heap_free_all;
}

#if USE_RAW && (RPMALLOC_FIRST_CLASS_HEAPS) {
    heap_realloc :: rpmalloc_heap_realloc;
}

#if USE_RAW && (RPMALLOC_FIRST_CLASS_HEAPS) {
    heap_release :: rpmalloc_heap_release;
}

#if USE_RAW && (RPMALLOC_FIRST_CLASS_HEAPS) {
    heap_thread_set_current :: rpmalloc_heap_thread_set_current;
}

#if USE_RAW {
    initialize :: rpmalloc_initialize;
}

#if USE_RAW {
    initialize_config :: rpmalloc_initialize_config;
}

#if USE_RAW {
    is_initialized :: rpmalloc_is_initialized;
}

#if USE_RAW {
    is_thread_initialized :: rpmalloc_is_thread_initialized;
}

RPMALLOC_CONFIGURABLE :: CONFIGURABLE;

#if USE_RAW && (RPMALLOC_FIRST_CLASS_HEAPS) {
    rpmalloc_heap_t :: heap_t;
}

#if USE_RAW {
    set_main_thread :: rpmalloc_set_main_thread;
}

#if USE_RAW {
    thread_finalize :: rpmalloc_thread_finalize;
}

#if USE_RAW {
    thread_initialize :: rpmalloc_thread_initialize;
}

#if USE_RAW {
    thread_statistics :: rpmalloc_thread_statistics;
}

#if USE_RAW {
    usable_size :: rpmalloc_usable_size;
}

// Native storage

errno: enum s32 {
    SUCCESS :: 0;
    ENOMEM :: 12;
    EINVAL :: 22;
}
