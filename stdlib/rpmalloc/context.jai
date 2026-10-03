// Generated from complete reviewed rpmalloc contextual record contracts.

// Regenerate with tools/native_bindings/generate.py; do not hand-edit.

// Private canonical types; never cast native SDK heap/span storage.

#scope_module

#if !USE_RAW {

    Heap_Size_Class :: struct {
        free_list: *void;
        partial_span: *Span;
        cache: *Span;
    }

    Span_Cache :: struct {
        count: s64;
        span: [400] *Span;
    }

    Span_Large_Cache :: struct {
        count: s64;
        span: [100] *Span;
    }

    Span_Use :: struct {
        current: s32;
        high: s32;
        #if ENABLE_STATISTICS {
            spans_deferred: s32;
            spans_to_global: s32;
            spans_from_global: s32;
            spans_to_cache: s32;
            spans_from_cache: s32;
            spans_to_reserved: s32;
            spans_from_reserved: s32;
            spans_map_calls: s32;
        }
    }

    #if ENABLE_STATISTICS {
        Size_Class_Use :: struct {
            alloc_current: s32;
            alloc_peak: s32;
            alloc_total: s32;
            free_total: s32;
            spans_current: s32;
            spans_peak: s32;
            spans_to_cache: s32;
            spans_from_cache: s32;
            spans_from_reserved: s32;
            spans_map_calls: s32;
            unused: s32;
        }
    }

    Span :: struct {
        free_list: *void;
        block_count: u32;
        size_class: u32;
        free_list_limit: u32;
        used_count: u32;
        list_size: u32;
        block_size: u32;
        span_count: u32;
        total_spans: u32;
        offset_from_master: u32;
        remaining_spans: s32;
        align_offset: u32;
        free_list_deferred: *void;
        flags: enum_flags u32 {
            MASTER :: 1;
            SUBSPAN :: 2;
            ALIGNED_BLOCKS :: 4;
            UNMAPPED_MASTER :: 8;
        };
        heap: *Heap;
        next: *Span;
        prev: *Span;
    }

    Heap :: struct {
        owner_thread: u64;
        size_class: [126] Heap_Size_Class;
        span_free_deferred: *void;
        full_span_count: s64;
        span_reserve: *Span;
        span_reserve_master: *Span;
        spans_reserved: u32;
        child_count: s32;
        next_heap: *Heap;
        next_orphan: *Heap;
        id: s32;
        finalize: s32;
        master_heap: *Heap;
        full_span: [126] *Span;
        large_huge_span: *Span;
        #if ENABLE_THREAD_CACHE {
            span_cache: Span_Cache;
            span_large_cache: [62] Span_Large_Cache;
        }
        #if ENABLE_ADAPTIVE_THREAD_CACHE || ENABLE_STATISTICS {
            span_use: [63] Span_Use;
        }
        #if ENABLE_STATISTICS {
            size_class_use: [127] Size_Class_Use;
            thread_to_global: s64;
            global_to_thread: s64;
        }
    }

}

#if USE_RAW {

    size_t :: u64;
    _int :: s32;
    int32_t :: s32;

    atomic32_t :: s32;
    atomic64_t :: s64;
    atomicptr_t :: *void;

    // Explicit private build selection; no undeclared public import option.

    Raw_Build_Configuration :: #import "internal/bindings/rpmalloc-build";

    RPMALLOC_FIRST_CLASS_HEAPS :: Raw_Build_Configuration.FIRST_CLASS_HEAPS;

    heap_size_class_t :: struct {
        free_list: *void;
        partial_span: *span_t;
        cache: *span_t;
    }

    span_cache_t :: struct {
        count: size_t;
        span: [400] *span_t;
    }

    span_large_cache_t :: struct {
        count: size_t;
        span: [100] *span_t;
    }

    span_use_t :: struct {
        current: atomic32_t;
        high: int32_t;
        #if ENABLE_STATISTICS {
            spans_deferred: atomic32_t;
            spans_to_global: atomic32_t;
            spans_from_global: atomic32_t;
            spans_to_cache: atomic32_t;
            spans_from_cache: atomic32_t;
            spans_to_reserved: atomic32_t;
            spans_from_reserved: atomic32_t;
            spans_map_calls: atomic32_t;
        }
    }

    #if ENABLE_STATISTICS {
        size_class_use_t :: struct {
            alloc_current: atomic32_t;
            alloc_peak: int32_t;
            alloc_total: atomic32_t;
            free_total: atomic32_t;
            spans_current: atomic32_t;
            spans_peak: int32_t;
            spans_to_cache: atomic32_t;
            spans_from_cache: atomic32_t;
            spans_from_reserved: atomic32_t;
            spans_map_calls: atomic32_t;
            unused: atomic32_t;
        }
    }

    span_t :: struct {
        free_list: *void;
        block_count: u32;
        size_class: u32;
        free_list_limit: u32;
        used_count: u32;
        list_size: u32;
        block_size: u32;
        span_count: u32;
        total_spans: u32;
        offset_from_master: u32;
        remaining_spans: atomic32_t;
        align_offset: u32;
        free_list_deferred: atomicptr_t;
        flags: u32;
        heap: *heap_t;
        next: *span_t;
        prev: *span_t;
    }

    heap_t :: struct {
        owner_thread: u64;
        size_class: [126] heap_size_class_t;
        span_free_deferred: atomicptr_t;
        full_span_count: size_t;
        span_reserve: *span_t;
        span_reserve_master: *span_t;
        spans_reserved: u32;
        child_count: atomic32_t;
        next_heap: *heap_t;
        next_orphan: *heap_t;
        id: int32_t;
        finalize: _int;
        master_heap: *heap_t;
        #if ENABLE_THREAD_CACHE {
            span_cache: span_cache_t;
            span_large_cache: [62] span_large_cache_t;
        }
        #if RPMALLOC_FIRST_CLASS_HEAPS {
            full_span: [126] *span_t;
            large_huge_span: *span_t;
        }
        #if ENABLE_ADAPTIVE_THREAD_CACHE || ENABLE_STATISTICS {
            span_use: [63] span_use_t;
        }
        #if ENABLE_STATISTICS {
            size_class_use: [127] size_class_use_t;
            thread_to_global: atomic64_t;
            global_to_thread: atomic64_t;
        }
    }

}
