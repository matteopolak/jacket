// Authored consumer: block headers, accounting, resize and retained blocks.
#import "Basic";
P :: #import "Pool";
#load "storage-allocator-tracker.jai";

Pool_Layout :: struct {
    memblock_size: s64 = 65536;
    oversized_size: s64 = 6554;
    alignment := 8;
    current_memblock: *void;
    current_pos: *u8;
    bytes_left: s64;
    block_allocator: Allocator;
    unused_memblocks: *void;
    used_memblocks: *void;
    first_used_memblock: *void;
    out_of_band_allocations: *void;
    overwrite_memory := false;
    free_memblocks_on_reset := false;
    logging_options: enum_flags u32 {
        LOG_OVERSIZED_ALLOCATIONS :: 0x1;
        LOG_OVERSIZED_ALLOCATIONS_AS_ERRORS :: 0x2;
        LOG_OVERSIZED_ALLOCATIONS_STACK_TRACE :: 0x4;
    }
}
#assert size_of(P.Pool) == size_of(Pool_Layout);
#assert P.HEADER_SIZE == 8;

check_pool_contract :: () -> int {
    if !storage_same_layout(P.Pool, Pool_Layout) return 1;
    pool: P.Pool;
    if pool.memblock_size != 65536 || pool.oversized_size != 6554 || pool.alignment != 8 || pool.block_allocator.proc || pool.overwrite_memory || pool.free_memblocks_on_reset return 2;
    pointer_type := cast(*Type_Info_Pointer)type_of(pool.current_memblock);
    header_type := cast(*Type_Info_Struct)pointer_type.pointer_to;
    if header_type.runtime_size != P.HEADER_SIZE || header_type.members.count != 2 return 3;
    if header_type.members[0].name != "next" || header_type.members[0].offset_in_bytes != 0 || header_type.members[1].name != "data" || header_type.members[1].offset_in_bytes != 8 || header_type.members[1].type.runtime_size != 0 return 4;
    allocator := Allocator.{P.pool_allocator_proc, *pool};
    caps, name := get_capabilities_info(allocator);
    if name != "modules/Pool" || (caps & .IS_THIS_YOURS) == 0 || (caps & .HINT_I_AM_A_FAST_BUMP_ALLOCATOR) == 0 return 5;
    allocator.proc(.STARTUP, 0, 0, null, allocator.data);
    if pool.block_allocator.proc return 6;

    trace: Storage_Allocation_Trace;
    backing := storage_trace_allocator(*trace);
    pool.memblock_size = 64;
    pool.oversized_size = 128;
    pool.overwrite_memory = true;
    P.set_allocators(*pool, backing);
    defer P.release(*pool);
    first := cast(*u8)P.get(*pool, 16);
    first_block := pool.current_memblock;
    if !first || first != cast(*u8)first_block + P.HEADER_SIZE || pool.bytes_left != 40 || first[0] != 204 return 7;
    used, oversized := P.calculate_used_memory(pool);
    if used != 24 || oversized != 0 return 8;
    first[0] = 37;
    second := cast(*u8)P.get(*pool, 48);
    if !second || second == first || pool.current_memblock.next != first_block || pool.first_used_memblock != first_block || pool.bytes_left != 8 return 9;
    used, oversized = P.calculate_used_memory(pool);
    if used != 120 || oversized != 0 return 10;
    large := cast(*u8)P.get(*pool, 128);
    if !large || !pool.out_of_band_allocations || ((cast(u64)large) & 7) != 0 return 11;
    used, oversized = P.calculate_used_memory(pool);
    if used != 120 || oversized != 1 return 12;
    smaller := cast(*u8)allocator.proc(.RESIZE, 8, 16, first, allocator.data);
    if !smaller || smaller == first || smaller[0] != 37 || pool.bytes_left != 0 return 13;
    grown := cast(*u8)allocator.proc(.RESIZE, 32, 8, smaller, allocator.data);
    if !grown || grown == smaller || grown[0] != 37 || pool.bytes_left != 24 return 14;
    used, oversized = P.calculate_used_memory(pool);
    if used != 168 || oversized != 1 || trace.allocations != 4 return 15;
    allocator.proc(.FREE, 0, 0, first, allocator.data);
    allocator.proc(.SHUTDOWN, 0, 0, null, allocator.data);
    if first[0] != 37 || trace.frees != 0 || !allocator.proc(.IS_THIS_YOURS, 0, 0, grown, allocator.data) || !allocator.proc(.IS_THIS_YOURS, 0, 0, large, allocator.data) return 16;
    P.reset(*pool);
    if trace.frees != 1 || trace.outstanding != 3 || pool.used_memblocks || pool.first_used_memblock || pool.current_memblock || pool.current_pos || pool.bytes_left != 0 || pool.out_of_band_allocations return 17;
    if first[0] != 204 || second[0] != 204 || smaller[0] != 204 || grown[0] != 204 || P.get(*pool, 16) != grown return 18;
    P.release(*pool);
    if trace.outstanding != 0 || trace.frees != 4 || pool.unused_memblocks || pool.used_memblocks || pool.current_pos return 19;
    used, oversized = P.calculate_used_memory(pool);
    if used != 0 || oversized != 0 return 20;
    return 0;
}

#assert #run check_pool_contract() == 0;
main :: () -> int { return check_pool_contract(); }
