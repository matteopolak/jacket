// Authored allocation evidence helper; forwards real allocations to the heap.
Storage_Allocation_Trace :: struct {
    backing: Allocator;
    allocations: int;
    frees: int;
    outstanding: int;
    limit: int = -1;
}

storage_trace_allocator :: (trace: *Storage_Allocation_Trace) -> Allocator {
    trace.backing = context.default_allocator;
    return .{storage_trace_proc, trace};
}

storage_trace_proc :: (mode: Allocator_Mode, size: s64, old_size: s64, old_memory: *void, data: *void) -> *void {
    trace := cast(*Storage_Allocation_Trace)data;
    if mode == .ALLOCATE && trace.limit >= 0 && trace.allocations >= trace.limit return null;
    result := trace.backing.proc(mode, size, old_size, old_memory, trace.backing.data);
    if mode == .ALLOCATE && result { trace.allocations += 1; trace.outstanding += 1; }
    if mode == .FREE && old_memory { trace.frees += 1; trace.outstanding -= 1; }
    return result;
}

storage_same_layout :: ($A: Type, $B: Type) -> bool {
    if size_of(A) != size_of(B) return false;
    actual := cast(*Type_Info_Struct)A;
    expected := cast(*Type_Info_Struct)B;
    if actual.members.count != expected.members.count return false;
    for index: 0..actual.members.count - 1 {
        a := actual.members[index];
        b := expected.members[index];
        if a.name != b.name || a.offset_in_bytes != b.offset_in_bytes || a.type.runtime_size != b.type.runtime_size return false;
    }
    return true;
}
