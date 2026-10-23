// Allocation patterns: the default heap, the temporary allocator, Pool and Flat_Pool.
#import "Basic";
#import "Pool";
#import "Flat_Pool";

main :: () {
    checksum: u64 = 1469598103934665603;

    mix :: (value: int) #expand {
        checksum = (checksum ^ cast,no_check(u64) value) * 1099511628211;
    }

    // Default heap: a sliding window of live blocks of mixed sizes.
    WINDOW :: 1024;
    live: [WINDOW] *u8;
    state: u32 = 17;

    for 0..2_000_000 - 1 {
        slot := it % WINDOW;
        if live[slot] free(live[slot]);

        state = state * 1664525 + 1013904223;
        size := 8 + (state >> 20) % 512;

        live[slot] = alloc(cast(int) size);
        live[slot][0] = cast(u8) it;
        mix(live[slot][0]);
    }

    for live free(it);

    // Temporary storage: many small pieces, reset every so often.
    for 0..3_000_000 - 1 {
        piece := cast(*u8) talloc(16 + it % 48);
        piece[0] = cast(u8) it;
        mix(piece[0]);
        if it % 1000 == 999 reset_temporary_storage();
    }

    // Pool and Flat_Pool through their allocators.
    pool: Pool;

    set_allocators(*pool);
    flat: Flat_Pool;

    for round: 0..39 {
        push_allocator(pool_allocator_proc, *pool);

        for 0..50_000 - 1 {
            piece := cast(*u8) alloc(24 + it % 40);
            piece[0] = cast(u8) it;
            mix(piece[0]);
        }

        reset(*pool);
    }

    for round: 0..39 {
        push_allocator(flat_pool_allocator_proc, *flat);

        for 0..50_000 - 1 {
            piece := cast(*u8) alloc(24 + it % 40);
            piece[0] = cast(u8) it;
            mix(piece[0]);
        }

        reset(*flat);
    }

    release(*pool);
    fini(*flat);

    print("%\n", checksum);
}
