// Real memory, a bounded source allocator, and exact failure-state assertions.
#load "../../stdlib/Basic/Memory_Debugger_Core.jai";

arena: [65536] u64;
arena_position: s64;
release_count: s64;
fail_allocations: bool;
arena_allocator :: (mode: Allocator_Mode, size: s64, old_size: s64, old_memory: *void, data: *void) -> *void {
    if mode == .FREE {
        release_count += 1;
        return null;
    }
    if fail_allocations return null;
    if mode != .ALLOCATE || size < 0 return null;
    bytes := (size + 7) & ~7;
    if bytes > size_of(arena) - arena_position return null;
    result := cast(*u8) *arena + arena_position;
    arena_position += bytes;
    return result;
}

main :: () -> s64 {
    state: MD_State;
    allocator := Allocator.{ arena_allocator, null };
    state.storage_allocator = allocator;
    defer md_deinit(*state);
    one: [16] u8;
    two: [24] u8;
    unknown: u8;
    metadata: Stack_Trace_Procedure_Info;
    metadata.name = "original-caller";
    stack := Stack_Trace_Node.{ null, *metadata, 42, 1, 12 };
    if md_alloc(*state, allocator, *one, 16, *stack, 7) != .OK return 1;
    first := md_find(*state, *one);
    if !first || !first.allocation.is_live || first.allocation.index != 1 || first.allocation.thread_index != 7 return 2;
    if first.allocation.alloc_site_trace.nodes.count != 1 || first.allocation.alloc_site_trace.nodes[0].line_number != 12 return 3;
    stack.line_number = 99;
    if first.allocation.alloc_site_trace.nodes[0].line_number != 12 return 4;
    if md_alloc(*state, allocator, *one, 16, null, 0) != .ALREADY_LIVE return 5;
    other_allocator := Allocator.{ arena_allocator, *unknown };
    if md_free(*state, other_allocator, *one, null) != .ALLOCATOR_MISMATCH || !first.allocation.is_live return 6;
    if md_realloc(*state, allocator, *one, 15, *two, 24, null, 0) != .SIZE_MISMATCH || !first.allocation.is_live return 7;
    if md_realloc(*state, allocator, *one, 16, null, 24, null, 0) != .FAILED_REALLOC || !first.allocation.is_live return 8;
    if md_realloc(*state, allocator, *one, 16, *one, 8, null, 0) != .OK || first.allocation.size != 8 return 9;
    if first.frame_frees != 1 || first.frame_freed_bytes != 8 return 21;
    first.ignored = true;
    first.allocation.propagates_leak_ownership = false;
    if md_realloc(*state, allocator, *one, 8, *two, 24, *stack, 8) != .OK return 10;
    moved := md_find(*state, *two);
    if first.allocation.is_live || !moved || !moved.allocation.is_live || moved.allocation.index != 2 return 11;
    if !moved.ignored || moved.allocation.propagates_leak_ownership || moved.allocation.thread_index != 8 return 12;
    if md_free(*state, allocator, *one, null) != .ALREADY_FREED return 13;
    if md_free(*state, allocator, *unknown, null) != .UNKNOWN_MEMORY return 14;
    if md_free(*state, allocator, null, null) != .OK return 15;
    moved.ignored = false;
    if !md_is_leak(*state, moved) return 16;
    state.first_report_index = state.next_index;
    if md_is_leak(*state, moved) return 17;
    if md_destroy_heap(*state, other_allocator, null) != .OK || !moved.allocation.is_live return 18;
    if md_destroy_heap(*state, allocator, null) != .OK || moved.allocation.is_live return 19;
    if md_alloc(*state, allocator, *one, 16, null, 0) != .OK || md_alloc(*state, allocator, *two, 24, null, 0) != .OK return 22;
    fail_allocations = true;
    if md_destroy_heap(*state, allocator, null) != .OUT_OF_MEMORY return 23;
    if md_find(*state, *one).allocation.is_live || md_find(*state, *two).allocation.is_live return 24;
    fail_allocations = false;
    if md_alloc(*state, allocator, *one, 16, null, 0) != .OK return 25;
    fail_allocations = true;
    if md_realloc(*state, allocator, *one, 16, *two, 24, null, 0) != .OUT_OF_MEMORY || md_find(*state, *one).allocation.is_live return 26;
    fail_allocations = false;
    before := release_count;
    md_deinit(*state);
    if state.records != null || state.traces != null || state.next_index != 1 || release_count <= before return 20;
    return 0;
}
