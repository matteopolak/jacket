// Authored consumer: historical record, one captured backing allocation, pointer cursor.
#import "Basic";
P :: #import "legacy/Flat_Pool"(DEFAULT_VIRTUAL_MEMORY_RESERVE=513);
#load "storage-allocator-tracker.jai";

Legacy_Flat_Pool_Layout :: struct {
    alignment: s32 = 8;
    current_point: *u8;
    memory_base: *u8;
    first_uncommitted_page: *u8;
    address_limit: *u8;
}
#assert size_of(P.Flat_Pool) == size_of(Legacy_Flat_Pool_Layout);

check_legacy_flat_pool_contract :: () -> int {
    if !storage_same_layout(P.Flat_Pool, Legacy_Flat_Pool_Layout) return 1;
    pool: P.Flat_Pool;
    if pool.alignment != 8 || pool.current_point || pool.memory_base || pool.first_uncommitted_page || pool.address_limit return 2;
    allocator := Allocator.{P.flat_pool_allocator_proc, *pool};
    caps, name := get_capabilities_info(allocator);
    if name != "modules/Flat_Pool" || (caps & .IS_THIS_YOURS) == 0 || (caps & .HINT_I_AM_A_FAST_BUMP_ALLOCATOR) == 0 return 3;
    allocator.proc(.STARTUP, 0, 0, null, allocator.data);
    if pool.memory_base != null || allocator.proc(.IS_THIS_YOURS, 0, 0, *pool, allocator.data) != null return 4;
    trace: Storage_Allocation_Trace;
    backing := storage_trace_allocator(*trace);
    pool.alignment = 64;
    {
        push_allocator(backing);
        P.init(*pool, 513);
    }
    defer P.fini(*pool);
    if trace.allocations != 1 || trace.outstanding != 1 || pool.address_limit - pool.memory_base != 4096 return 5;
    if pool.current_point != pool.memory_base || pool.first_uncommitted_page != pool.address_limit || (cast(u64)pool.memory_base & 63) != 0 return 6;
    first := cast(*u8)P.get(*pool, 16);
    second := cast(*u8)P.get(*pool, 16);
    if first != pool.memory_base || second != first + 64 || pool.current_point != first + 80 return 7;
    first[0] = 37;
    second[0] = 91;
    pool.memory_base[300] = 17;
    smaller := cast(*u8)allocator.proc(.RESIZE, 8, 16, first, allocator.data);
    if !smaller || smaller == first || smaller != first + 128 || smaller[0] != 37 return 8;
    larger := cast(*u8)allocator.proc(.RESIZE, 32, 8, smaller, allocator.data);
    if !larger || larger == smaller || larger != first + 192 || larger[0] != 37 return 9;
    allocator.proc(.FREE, 0, 0, first, allocator.data);
    allocator.proc(.SHUTDOWN, 0, 0, null, allocator.data);
    if pool.current_point != first + 224 || trace.frees != 0 || first[0] != 37 return 10;
    if !allocator.proc(.IS_THIS_YOURS, 0, 0, first, allocator.data) || !allocator.proc(.IS_THIS_YOURS, 0, 0, pool.address_limit - 1, allocator.data) return 11;
    if allocator.proc(.IS_THIS_YOURS, 0, 0, pool.address_limit, allocator.data) return 12;
    P.reset(*pool, true);
    if pool.current_point != pool.memory_base || first[0] != 204 || second[0] != 204 || smaller[0] != 204 || larger[0] != 204 || pool.memory_base[300] != 17 return 13;
    if P.get(*pool, 0) != first || pool.current_point != first return 14;
    other_trace: Storage_Allocation_Trace;
    other_backing := storage_trace_allocator(*other_trace);
    {
        push_allocator(other_backing);
        P.fini(*pool);
    }
    if trace.outstanding != 0 || trace.frees != 1 || other_trace.frees != 0 || pool.alignment != 64 return 15;
    if pool.current_point || pool.memory_base || pool.first_uncommitted_page || pool.address_limit return 16;
    P.fini(*pool);
    if trace.frees != 1 return 17;

    lazy: P.Flat_Pool;
    lazy_allocator := Allocator.{P.flat_pool_allocator_proc, *lazy};
    {
        push_allocator(lazy_allocator);
        memory := alloc(24);
        if !memory || memory != lazy.memory_base || lazy.current_point != lazy.memory_base + 24 || lazy.address_limit - lazy.memory_base != 4096 return 18;
    }
    lazy_allocator.proc(.SHUTDOWN, 0, 0, null, lazy_allocator.data);
    if !lazy.memory_base return 19;
    P.fini(*lazy);
    if lazy.memory_base || lazy.current_point return 20;
    return 0;
}

#assert #run check_legacy_flat_pool_contract() == 0;
main :: () -> int { return check_legacy_flat_pool_contract(); }
