// Generated from complete pinned official rpmalloc C source, SDK build profile.
// Storage layout only: these scalar views do not implement atomic operations.
// Never alias reviewed public Jai heaps/spans to these native records.
#assert size_of(*void) == 8, "Verified native storage requires 64-bit pointers";

Size_Class_Storage :: struct {
    free_list: *void;
    partial_span: *Span_Storage;
    cache: *Span_Storage;
}

Span_Cache_Storage :: struct {
    count: u64;
    span: [400] *Span_Storage;
}

Large_Cache_Storage :: struct {
    count: u64;
    span: [100] *Span_Storage;
}

Span_Counters_Storage :: struct {
    current: s32;
    high: s32;
    spans_deferred: s32;
    spans_to_global: s32;
    spans_from_global: s32;
    spans_to_cache: s32;
    spans_from_cache: s32;
    spans_to_reserved: s32;
    spans_from_reserved: s32;
    spans_map_calls: s32;
}

Size_Counters_Storage :: struct {
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

Span_Storage :: struct {
    free_list: *void;
    block_count: u32;
    size_class: u32;
    free_list_limit: u32;
    used_count: u32;
    free_list_deferred: *void;
    list_size: u32;
    block_size: u32;
    flags: u32;
    span_count: u32;
    total_spans: u32;
    offset_from_master: u32;
    remaining_spans: s32;
    align_offset: u32;
    heap: *Heap_Storage;
    next: *Span_Storage;
    prev: *Span_Storage;
}

Heap_Storage :: struct {
    owner_thread: u64;
    size_class: [126] Size_Class_Storage;
    span_cache: Span_Cache_Storage;
    span_free_deferred: *void;
    full_span_count: u64;
    span_reserve: *Span_Storage;
    span_reserve_master: *Span_Storage;
    spans_reserved: u32;
    child_count: s32;
    next_heap: *Heap_Storage;
    next_orphan: *Heap_Storage;
    id: s32;
    finalize: s32;
    master_heap: *Heap_Storage;
    span_large_cache: [62] Large_Cache_Storage;
    full_span: [126] *Span_Storage;
    large_huge_span: *Span_Storage;
    span_use: [63] Span_Counters_Storage;
    size_class_use: [127] Size_Counters_Storage;
    thread_to_global: s64;
    global_to_thread: s64;
}
