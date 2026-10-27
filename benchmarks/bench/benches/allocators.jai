// Allocation patterns: the default heap, the temporary allocator, Pool and Flat_Pool.
#import "Basic";
#import "Pool";
#import "Flat_Pool";
#import "Extensions/Testing";

// A sliding window of 1 024 live blocks of mixed sizes, 2 000 000 allocations.
heap_window :: (b: *Bench) {
    set_items(b, 2_000_000);

    for 0..b.iterations - 1 {
        WINDOW :: 1024;
        live: [WINDOW] *u8;
        state: u32 = 17;
        checksum: u64 = 1469598103934665603;

        for 0..2_000_000 - 1 {
            slot := it % WINDOW;
            if live[slot] free(live[slot]);

            state = state * 1664525 + 1013904223;
            size := 8 + (state >> 20) % 512;

            live[slot] = alloc(cast(int) size);
            live[slot][0] = cast(u8) it;
            checksum = (checksum ^ cast,no_check(u64) live[slot][0]) * 1099511628211;
        }

        for live free(it);

        black_box(checksum);
    }
} @bench @samples(20)

// 3 000 000 small pieces of temporary storage, reset every 1 000.
temporary_storage :: (b: *Bench) {
    set_items(b, 3_000_000);

    for 0..b.iterations - 1 {
        checksum: u64 = 1469598103934665603;

        for 0..3_000_000 - 1 {
            piece := cast(*u8) talloc(16 + it % 48);
            piece[0] = cast(u8) it;
            checksum = (checksum ^ cast,no_check(u64) piece[0]) * 1099511628211;
            if it % 1000 == 999 reset_temporary_storage();
        }

        reset_temporary_storage();
        black_box(checksum);
    }
} @bench @samples(20)

// 40 rounds of 50 000 allocations through the Pool, which is reset after each round.
pool_allocations :: (b: *Bench) {
    set_items(b, 2_000_000);

    for 0..b.iterations - 1 {
        checksum: u64 = 1469598103934665603;
        pool: Pool;

        set_allocators(*pool);

        for round: 0..39 {
            push_allocator(pool_allocator_proc, *pool);

            for 0..50_000 - 1 {
                piece := cast(*u8) alloc(24 + it % 40);
                piece[0] = cast(u8) it;
                checksum = (checksum ^ cast,no_check(u64) piece[0]) * 1099511628211;
            }

            reset(*pool);
        }

        release(*pool);
        black_box(checksum);
    }
} @bench @samples(20)

// The same 40 rounds through a Flat_Pool.
flat_pool_allocations :: (b: *Bench) {
    set_items(b, 2_000_000);

    for 0..b.iterations - 1 {
        checksum: u64 = 1469598103934665603;
        flat: Flat_Pool;

        for round: 0..39 {
            push_allocator(flat_pool_allocator_proc, *flat);

            for 0..50_000 - 1 {
                piece := cast(*u8) alloc(24 + it % 40);
                piece[0] = cast(u8) it;
                checksum = (checksum ^ cast,no_check(u64) piece[0]) * 1099511628211;
            }

            reset(*flat);
        }

        fini(*flat);
        black_box(checksum);
    }
} @bench @samples(20)
