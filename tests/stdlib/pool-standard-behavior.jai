// Checks Pool against the standard module's observable behavior: lazy defaults,
// alignment, byte accounting, the oversized path, reset/release, and context use.
#import "Basic";
#import "Pool";

Sample :: struct { a: s64 = 11; b: s64 = 22; }

main :: () -> s64 {
    // Defaults before any use.
    p: Pool;
    if p.memblock_size != 65536 || p.oversized_size != 6554 || p.alignment != 8 return 1;
    if p.bytes_left != 0 || p.current_memblock != null || p.block_allocator.proc != null return 2;

    // First get lazily picks the context allocator and takes one block.
    a := cast(*u8) get(*p, 10);
    if a == null return 3;
    if p.block_allocator.proc != context.allocator.proc return 4;
    if p.current_memblock == null || p.used_memblocks != p.current_memblock return 5;
    if p.bytes_left != 65536 - 8 - 10 return 6;
    if ((cast(u64) a) & 7) != 0 return 7;

    // Next allocation is padded to the alignment.
    b := cast(*u8) get(*p, 8);
    if ((cast(u64) b) & 7) != 0 return 8;
    if b != a + 16 return 9;
    if p.bytes_left != 65536 - 8 - 10 - 6 - 8 return 10;

    // Custom alignment.
    q: Pool;
    q.alignment = 32;
    x := get(*q, 3);
    y := get(*q, 3);
    if ((cast(u64) x) & 31) != 0 || ((cast(u64) y) & 31) != 0 || y == x return 11;

    // Oversized: separate allocation, tracked out of band, current block untouched.
    left := p.bytes_left;
    pos := p.current_pos;
    block := p.current_memblock;
    big := cast(*u8) get(*p, 6554);
    if big == null return 12;
    if ((cast(u64) big) & 7) != 0 return 13;
    if p.out_of_band_allocations == null return 14;
    if p.bytes_left != left || p.current_pos != pos || p.current_memblock != block return 15;
    for 0..6553 big[it] = 7;
    used, outband := calculate_used_memory(p);
    if outband != 1 || used != 65536 - p.bytes_left return 16;
    just_under := get(*p, 6553);
    if just_under == null || p.out_of_band_allocations.next != null return 17;

    // Filling a block moves on to a fresh one.
    first_block := p.current_memblock;
    for 0..20 get(*p, 6000);
    if p.current_memblock == first_block || p.used_memblocks == first_block return 18;
    if p.first_used_memblock != first_block return 19;

    // Reset: blocks are kept for reuse, oversized memory is freed.
    p.memblock_size = 65536;
    reset(*p);
    if p.used_memblocks != null || p.unused_memblocks == null return 20;
    if p.out_of_band_allocations != null || p.bytes_left != 0 || p.current_pos != null return 21;
    again := cast(*u8) get(*p, 4);
    if again == null || p.bytes_left != 65536 - 8 - 4 return 22;
    if p.unused_memblocks == null return 23;   // still holds blocks it hasn't needed yet

    // Release: everything goes back.
    release(*p);
    if p.used_memblocks != null || p.unused_memblocks != null || p.out_of_band_allocations != null return 24;
    if p.bytes_left != 0 || p.current_memblock != null return 25;
    after := get(*p, 1);   // still usable afterwards
    if after == null return 26;
    release(*p);

    // As the context allocator.
    r: Pool;
    set_allocators(*r);
    {
        push_allocator(Allocator.{pool_allocator_proc, *r});
        s := New(Sample);
        if s == null || s.a != 11 || s.b != 22 return 27;
        buffer := alloc(24);
        if buffer == null return 28;
        free(buffer);   // ignored by the pool
        grown := cast(*u8) realloc(buffer, 48, 24);
        if grown == null return 29;
    }
    if r.used_memblocks == null || r.bytes_left >= 65536 - 8 return 30;
    caller := Allocator.{pool_allocator_proc, *r};
    if !caller.proc(.IS_THIS_YOURS, 0, 0, r.current_pos - 4, caller.data) return 31;
    release(*r);
    if r.used_memblocks != null return 32;

    print("pool-standard-behavior ok\n");
    return 0;
}
