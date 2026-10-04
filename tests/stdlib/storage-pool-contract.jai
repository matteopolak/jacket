// Authored consumers: exact record, real backing ownership, context allocator use.
#import "Basic";
P :: #import "Pool";
#load "storage-allocator-tracker.jai";

Pool_Layout :: struct {
    memblock_size: int;
    bytes_left: int;
    current_block: *void;
    current_pos: int;
}
#assert size_of(P.Pool) == size_of(Pool_Layout);

Pool_Thing :: struct { number: int = 23; label: string = "default"; }

check_pool_contract :: () -> int {
    if !storage_same_layout(P.Pool, Pool_Layout) return 1;
    initial: P.Pool;
    if initial.memblock_size != 0 || initial.bytes_left != 0 || initial.current_block != null || initial.current_pos != 0 return 2;
    before := Allocator.{P.pool_allocator_proc, *initial};
    caps, name := get_capabilities_info(before);
    if name != "modules/Pool" || (caps & .IS_THIS_YOURS) == 0 || (caps & .HINT_I_AM_A_FAST_BUMP_ALLOCATOR) == 0 return 3;
    if initial.current_block != null || (caps & .CREATE_HEAP) != 0 || (caps & .MULTIPLE_THREADS) != 0 return 4;

    trace: Storage_Allocation_Trace;
    backing := storage_trace_allocator(*trace);
    pool: P.Pool;
    P.set_allocators(*pool, backing);
    defer P.release(*pool);
    pool.memblock_size = 128;
    first := cast(*u8)P.get(*pool, 64);
    if !first || (cast(u64)first & 7) != 0 return 5;
    first[0] = 73;
    second := cast(*u8)P.get(*pool, 64);
    if !second || first == second || pool.bytes_left != 56 || pool.current_pos != 72 return 6;
    second[0] = 91;
    allocator := Allocator.{P.pool_allocator_proc, *pool};
    if !allocator.proc(.IS_THIS_YOURS, 0, 0, first, allocator.data) || !allocator.proc(.IS_THIS_YOURS, 0, 0, second, allocator.data) return 7;
    outside: u8;
    if allocator.proc(.IS_THIS_YOURS, 0, 0, *outside, allocator.data) return 8;
    allocator.proc(.FREE, 0, 0, first, allocator.data);
    if first[0] != 73 || second[0] != 91 return 9;
    smaller := allocator.proc(.RESIZE, 8, 64, first, allocator.data);
    if smaller != first return 10;
    larger := cast(*u8)allocator.proc(.RESIZE, 160, 64, first, allocator.data);
    if !larger || larger == first || larger[0] != 73 return 11;
    if allocator.proc(.ALLOCATE, -1, 0, null, allocator.data) != null return 12;
    P.reset(*pool);
    reused := cast(*u8)P.get(*pool, 64);
    if reused != first || reused[0] != 73 return 13;
    {
        push_allocator(allocator);
        thing := New(Pool_Thing);
        if !thing || thing.number != 23 || thing.label != "default" return 14;
        if !allocator.proc(.IS_THIS_YOURS, 0, 0, thing, allocator.data) return 15;
    }
    P.release(*pool);
    if trace.outstanding != 0 || trace.allocations != trace.frees || pool.current_block != null || pool.bytes_left != 0 || pool.current_pos != 0 return 16;

    limited: Storage_Allocation_Trace;
    limited.limit = 2;
    limited_backing := storage_trace_allocator(*limited);
    failed: P.Pool;
    P.set_allocators(*failed, limited_backing);
    failed.memblock_size = 128;
    saved := cast(*u8)P.get(*failed, 64);
    if !saved return 17;
    saved[0] = 31;
    if P.get(*failed, 64) != null || saved[0] != 31 return 18;
    P.reset(*failed);
    if P.get(*failed, 64) != saved return 19;
    P.release(*failed);
    if limited.outstanding != 0 return 20;

    lazy: P.Pool;
    lazy.memblock_size = 128;
    lazy_allocator := Allocator.{P.pool_allocator_proc, *lazy};
    {
        push_allocator(lazy_allocator);
        memory := alloc(16);
        if !memory || !lazy_allocator.proc(.IS_THIS_YOURS, 0, 0, memory, lazy_allocator.data) return 21;
    }
    lazy_allocator.proc(.SHUTDOWN, 0, 0, null, lazy_allocator.data);
    if lazy.current_block != null || lazy.bytes_left != 0 return 22;
    return 0;
}

#assert #run check_pool_contract() == 0;
main :: () -> int { return check_pool_contract(); }
