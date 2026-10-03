Global_Statistics :: struct {
    mapped: s64;
    mapped_peak: s64;
    cached: s64;
    huge_alloc: s64;
    huge_alloc_peak: s64;
    mapped_total: s64;
    unmapped_total: s64;
}

Heap_Statistics :: struct {
    sizecache: s64;
    spancache: s64;
}

Size_Class :: struct {
    block_size: u32;
    block_count: u16;
    class_idx: u16;
}

Heap_Size_Class :: struct {
    free_list: *void;
    partial_span: *Span;
    cache: *Span;
}

Span :: struct {
    free_list: *void;
    block_count: u32;
    size_class: u32;
    free_list_limit: u32;
    used_count: u32;
    free_list_deferred: *void;
    list_size: u32;
    block_size: u32;
    flags: enum_flags u32 {
        MASTER :: 1;
        SUBSPAN :: 2;
        ALIGNED_BLOCKS :: 4;
        UNMAPPED_MASTER :: 8;
    };
    span_count: u32;
    total_spans: u32;
    offset_from_master: u32;
    remaining_spans: s32;
    align_offset: u32;
    heap: *Heap;
    next: *Span;
    prev: *Span;
}

Heap :: struct {
    lock: *void;
    is_orphaned: bool;
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
}

#if OS == .WINDOWS {
    SYSTEM_INFO :: struct {
        _pad0: u32;
        dwPageSize: u32;
        _pad1: [8] u32;
        dwAllocationGranularity: u32;
        _pad2: u32;
    }
}
