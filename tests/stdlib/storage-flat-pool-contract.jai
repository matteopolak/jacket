// Authored consumers: exact record, aligned growing blocks, poison reset, release.
#import "Basic";
P :: #import "Flat_Pool";
#load "storage-allocator-tracker.jai";

Flat_Pool_Layout :: struct {
    memblock_size: int;
    bytes_left: int;
    current_block: *void;
    current_pos: int;
    alignment: int = 8;
}
#assert size_of(P.Flat_Pool) == size_of(Flat_Pool_Layout);

check_flat_pool_contract :: () -> int {
    if !storage_same_layout(P.Flat_Pool, Flat_Pool_Layout) return 1;
    initial: P.Flat_Pool;
    if initial.memblock_size != 0 || initial.bytes_left != 0 || initial.current_block != null || initial.current_pos != 0 || initial.alignment != 8 return 2;
    a := Allocator.{P.flat_pool_allocator_proc, *initial};
    caps, name := get_capabilities_info(a);
    if name != "modules/Flat_Pool" || (caps & .IS_THIS_YOURS) == 0 || (caps & .HINT_I_AM_A_FAST_BUMP_ALLOCATOR) == 0 return 3;
    if initial.current_block != null || (caps & .CREATE_HEAP) != 0 || (caps & .MULTIPLE_THREADS) != 0 return 4;

    trace: Storage_Allocation_Trace;
    backing := storage_trace_allocator(*trace);
    pool: P.Flat_Pool;
    pool.alignment = 64;
    {
        push_allocator(backing);
        P.init(*pool, 128);
    }
    defer P.fini(*pool);
    first := cast(*u8)P.get(*pool, 16);
    second := cast(*u8)P.get(*pool, 16);
    if !first || !second || (cast(u64)first & 63) != 0 || (cast(u64)second & 63) != 0 return 5;
    if second != first + 64 || pool.current_pos != 80 || pool.bytes_left != 48 return 6;
    first[0] = 71;
    second[0] = 99;
    third := cast(*u8)P.get(*pool, 80);
    if !third || third == first || third == second || (cast(u64)third & 63) != 0 return 7;
    third[0] = 53;
    allocator := Allocator.{P.flat_pool_allocator_proc, *pool};
    if !allocator.proc(.IS_THIS_YOURS, 0, 0, first, allocator.data) || !allocator.proc(.IS_THIS_YOURS, 0, 0, third, allocator.data) return 8;
    outside: u8;
    if allocator.proc(.IS_THIS_YOURS, 0, 0, *outside, allocator.data) return 9;
    allocator.proc(.FREE, 0, 0, first, allocator.data);
    if first[0] != 71 return 10;
    resized := cast(*u8)allocator.proc(.RESIZE, 160, 16, first, allocator.data);
    if !resized || resized == first || resized[0] != 71 return 11;
    if allocator.proc(.ALLOCATE, -1, 0, null, allocator.data) != null return 12;
    P.reset(*pool, true);
    if first[0] != 204 || second[0] != 204 || third[0] != 204 || resized[0] != 204 return 13;
    if P.get(*pool, 16) != first return 14;
    pool.alignment = 128;
    stricter := P.get(*pool, 16);
    if !stricter || (cast(u64)stricter & 127) != 0 return 15;
    P.fini(*pool);
    if trace.outstanding != 0 || trace.allocations != trace.frees || pool.current_block != null || pool.bytes_left != 0 || pool.current_pos != 0 || pool.memblock_size != 128 return 16;

    lazy: P.Flat_Pool;
    lazy.memblock_size = 64;
    lazy.alignment = 0;
    lazy_allocator := Allocator.{P.flat_pool_allocator_proc, *lazy};
    {
        push_allocator(lazy_allocator);
        memory := alloc(16);
        if !memory || lazy.alignment != 8 || !lazy_allocator.proc(.IS_THIS_YOURS, 0, 0, memory, lazy_allocator.data) return 17;
    }
    lazy_allocator.proc(.SHUTDOWN, 0, 0, null, lazy_allocator.data);
    if lazy.current_block != null || lazy.bytes_left != 0 return 18;
    return 0;
}

#assert #run check_flat_pool_contract() == 0;
main :: () -> int { return check_flat_pool_contract(); }
