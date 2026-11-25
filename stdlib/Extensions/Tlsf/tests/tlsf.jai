// The TLSF allocator: allocate, free, resize, alignment, reuse, merging, exhaustion, several pools, and threads.
#import "Basic";
#import "Extensions/Testing";
#import "Extensions/Tlsf";

allocates_and_frees :: () {
    storage: [8192] u8;
    tlsf: Tlsf;

    require(init_in_memory(*tlsf, storage.data, storage.count));

    a := cast(*s64) allocate(*tlsf, 100);
    b := cast(*s64) allocate(*tlsf, 200);

    require(a != null && b != null);
    expect(a != b);
    expect(allocation_size(a) >= 100 && allocation_size(b) >= 200);
    expect(owns(*tlsf, a) && owns(*tlsf, b));

    for 0..11 {
        a[it] = it;
        b[it] = -it;
    }

    for 0..11 {
        expect_equal(a[it], it);
        expect_equal(b[it], -it);
    }

    expect(check(*tlsf));

    release(*tlsf, a);
    release(*tlsf, b);
    release(*tlsf, null);
    expect(check(*tlsf));
    stats := statistics(*tlsf);

    expect_equal(stats.used_blocks, 0);
    expect_equal(stats.free_blocks, 1); // everything merged back into one block
} @test

blocks_are_16_byte_aligned :: () {
    storage: [16384] u8;
    tlsf: Tlsf;

    require(init_in_memory(*tlsf, (storage.data + 3), storage.count - 3)); // a misaligned pool still works

    for size: s64.[0, 1, 7, 16, 17, 33, 100, 1000] {
        block := allocate(*tlsf, size);
        require(block != null);
        expect_equal((cast(u64) block) % 16, 0);
    }

    expect(check(*tlsf));
} @test

aligned_allocations :: () {
    storage: [1 << 16] u8;
    tlsf: Tlsf;

    require(init_in_memory(*tlsf, storage.data, storage.count));

    blocks: [8] *void;

    for alignment, i: s64.[32, 64, 128, 256, 1024, 4096, 32, 512] {
        blocks[i] = allocate_aligned(*tlsf, 50 + i, alignment);
        require(blocks[i] != null);
        expect_equal((cast(u64) blocks[i]) % cast(u64) alignment, 0);
        expect(allocation_size(blocks[i]) >= 50 + i);
        expect(check(*tlsf));
    }

    for blocks release(*tlsf, it);

    expect(check(*tlsf));
    expect_equal(statistics(*tlsf).free_blocks, 1);
} @test

freed_memory_is_reused :: () {
    storage: [4096] u8;
    tlsf: Tlsf;

    require(init_in_memory(*tlsf, storage.data, storage.count));

    first := allocate(*tlsf, 300);

    release(*tlsf, first);
    expect(allocate(*tlsf, 300) == first);

    // A pool that fits two blocks keeps working under repeated allocate and free.
    for 0..999 {
        a := allocate(*tlsf, 1500);
        b := allocate(*tlsf, 1500);

        require(a != null && b != null);
        release(*tlsf, a);
        release(*tlsf, b);
    }

    expect(check(*tlsf));
} @test

neighbours_merge :: () {
    storage: [4096] u8;
    tlsf: Tlsf;

    require(init_in_memory(*tlsf, storage.data, storage.count));
    big := statistics(*tlsf).largest_free_payload;

    a := allocate(*tlsf, 500);
    b := allocate(*tlsf, 500);
    c := allocate(*tlsf, 500);

    require(a != null && b != null && c != null);

    release(*tlsf, a); // merges with nothing
    release(*tlsf, c); // merges with the free space behind it
    release(*tlsf, b); // merges both sides
    expect(check(*tlsf));
    expect_equal(statistics(*tlsf).free_blocks, 1);
    expect_equal(statistics(*tlsf).largest_free_payload, big);
    expect(allocate(*tlsf, big - big / 16) != null); // the search rounds requests up to the next size class
} @test

exhaustion_returns_null :: () {
    storage: [1024] u8;
    tlsf: Tlsf;

    require(init_in_memory(*tlsf, storage.data, storage.count));

    expect(allocate(*tlsf, 2000) == null);
    expect(allocate(*tlsf, -1) == null);

    count := 0;
    while allocate(*tlsf, 48) count += 1;

    expect(count >= 12);
    expect(allocate(*tlsf, 48) == null);
    expect(check(*tlsf));
} @test

resize_in_place_and_by_moving :: () {
    storage: [8192] u8;
    tlsf: Tlsf;

    require(init_in_memory(*tlsf, storage.data, storage.count));

    block := cast(*u8) allocate(*tlsf, 100);
    for 0..99 block[it] = cast(u8) it;

    grown := cast(*u8) resize(*tlsf, block, 2000); // free space follows: stays put

    expect(grown == block);
    for 0..99 expect_equal(grown[it], cast(u8) it);

    shrunk := cast(*u8) resize(*tlsf, grown, 40);

    expect(shrunk == block);
    for 0..39 expect_equal(shrunk[it], cast(u8) it);

    expect(check(*tlsf));

    // A block with a used neighbour behind it has to move.
    wall := allocate(*tlsf, 100);
    moved := cast(*u8) resize(*tlsf, shrunk, 3000);

    require(moved != null);
    expect(moved != block);
    for 0..39 expect_equal(moved[it], cast(u8) it);

    expect(check(*tlsf));

    // Failure keeps the old block valid.
    expect(resize(*tlsf, moved, 100000) == null);
    expect(owns(*tlsf, moved));
    expect(check(*tlsf));

    expect(resize(*tlsf, null, 64) != null); // null resizes like allocate
    release(*tlsf, wall);
    expect(check(*tlsf));
} @test

several_pools :: () {
    first: [1024] u8;
    second: [2048] u8;
    tlsf: Tlsf;

    require(init_in_memory(*tlsf, first.data, first.count));
    require(add_pool(*tlsf, second.data, second.count));

    big := allocate(*tlsf, 1500); // only the second pool is big enough

    require(big != null);
    expect(cast(*u8) big >= second.data && cast(*u8) big < second.data + second.count);
    expect(owns(*tlsf, big));

    small := allocate(*tlsf, 700);

    require(small != null);
    expect(cast(*u8) small >= first.data && cast(*u8) small < first.data + first.count);
    expect(check(*tlsf));
    pool_bytes := statistics(*tlsf).pool_bytes; // each pool loses up to 15 bytes to alignment

    expect(pool_bytes > 3000 && pool_bytes <= 3072);

    tiny: [32] u8;

    expect(!add_pool(*tlsf, tiny.data, tiny.count));
} @test

owned_pool_comes_from_the_backing_allocator :: () {
    tlsf: Tlsf;

    require(init(*tlsf, 4096));
    block := allocate(*tlsf, 100);

    expect_not_null(block);
    release(*tlsf, block);
    fini(*tlsf); // the test's leak tracking checks the pool went back

    expect(!init(*tlsf, 0));
    expect(!init(*tlsf, 10));
} @test

random_operations_keep_the_tables_consistent :: () {
    tlsf: Tlsf;

    require(init(*tlsf, 1 << 18));
    defer fini(*tlsf);

    live: [64] *void;
    sizes: [64] s64;
    seed: u64 = 12345;

    next :: (seed: *u64) -> u64 {
        seed.* = seed.* * 6364136223846793005 + 1442695040888963407;
        return seed.* >> 33;
    }

    for step: 0..19999 {
        slot := cast(s64) (next(*seed) % 64);
        size := cast(s64) (next(*seed) % 2000);

        if live[slot] == null {
            live[slot] = allocate(*tlsf, size);
            sizes[slot] = size;
            if live[slot] memset(live[slot], cast(u8) slot, size);
        } else if next(*seed) % 3 == 0 {
            moved := resize(*tlsf, live[slot], size);
            if moved {
                for 0..min(size, sizes[slot]) - 1 expect_equal((cast(*u8) moved)[it], cast(u8) slot);
                live[slot] = moved;
                sizes[slot] = size;
                memset(moved, cast(u8) slot, size);
            }
        } else {
            for 0..sizes[slot] - 1 expect_equal((cast(*u8) live[slot])[it], cast(u8) slot);
            release(*tlsf, live[slot]);
            live[slot] = null;
        }

        if step % 500 == 0 require(check(*tlsf));
    }

    for live release(*tlsf, it);

    expect(check(*tlsf));
    expect_equal(statistics(*tlsf).free_blocks, 1);
} @test

works_as_context_allocator :: () {
    storage: [1 << 16] u8;
    tlsf: Tlsf;

    require(init_in_memory(*tlsf, storage.data, storage.count));

    {
        push_allocator(as_allocator(*tlsf));
        numbers: [..] int;
        for 0..999 array_add(*numbers, it);

        expect_equal(numbers[999], 999);
        text := copy_string("allocated through the context");

        expect_equal(text, "allocated through the context");
        array_free(numbers);
        free(text);
    }

    expect_equal(statistics(*tlsf).used_blocks, 0);
    caps := cast(Allocator_Caps) cast(u64) tlsf_proc(.CAPS, 0, 0, null, *tlsf);

    expect(caps & .FREE != 0 && caps & .ACTUALLY_RESIZE != 0);
    expect(caps & .MULTIPLE_THREADS == 0);
} @test

// Wasm has no threads.
#if OS != .WASM {
    #import "Thread";

    Shared :: struct {
        tlsf: Tlsf;
        failures: s32;
    }

    thread_safe_instance_serves_many_threads :: () {
        shared: Shared;

        require(init(*shared.tlsf, 1 << 20, thread_safe = true));
        defer fini(*shared.tlsf);

        caps := cast(Allocator_Caps) cast(u64) tlsf_proc(.CAPS, 0, 0, null, *shared.tlsf);

        expect(caps & .MULTIPLE_THREADS != 0);

        threads: [4] Thread;
        for *threads {
            it.data = *shared;
            require(thread_init(it, (thread: *Thread) -> s64 {
                shared := cast(*Shared) thread.data;

                for 0..4999 {
                    size := 16 + (it % 200);
                    block := cast(*u8) tlsf_proc(.ALLOCATE, size, 0, null, *shared.tlsf);

                    if !block {
                        shared.failures += 1;
                        continue;
                    }

                    memset(block, cast(u8) size, size);
                    for 0..size - 1 if block[it] != cast(u8) size shared.failures += 1;

                    tlsf_proc(.FREE, 0, 0, block, *shared.tlsf);
                }

                return 0;
            }));
            thread_start(it);
        }

        for *threads {
            expect(thread_is_done(it, -1));
            thread_deinit(it);
        }

        expect_equal(shared.failures, 0);
        expect(check(*shared.tlsf));
        expect_equal(statistics(*shared.tlsf).used_blocks, 0);
    } @test
}
