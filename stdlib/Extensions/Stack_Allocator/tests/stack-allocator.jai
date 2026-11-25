// The stack allocator: allocate, release in and out of order, resize, alignment, reuse, marks, and threads.
#import "Basic";
#import "Extensions/Testing";
#import "Extensions/Stack_Allocator";

allocates_and_pops :: () {
    storage: [1024] u8;
    stack: Stack_Allocator;

    init_in_memory(*stack, storage.data, storage.count);

    a := cast(*u8) allocate(*stack, 100);
    b := cast(*u8) allocate(*stack, 100);

    require(a != null && b != null);
    expect(b >= a + 100);
    expect(owns(*stack, a) && owns(*stack, b));
    memset(a, 1, 100);
    memset(b, 2, 100);

    for 0..99 {
        expect_equal(a[it], 1);
        expect_equal(b[it], 2);
    }

    expect_equal(statistics(*stack).blocks, 2);

    expect(release(*stack, b));
    expect(!owns(*stack, b));
    expect(release(*stack, a));
    stats := statistics(*stack);

    expect_equal(stats.used, 0);
    expect_equal(stats.blocks, 0);
    expect(stats.high_water >= 200);
} @test

popped_memory_is_reused :: () {
    storage: [512] u8;
    stack: Stack_Allocator;

    init_in_memory(*stack, storage.data, storage.count);

    keep := allocate(*stack, 50);
    first := allocate(*stack, 100);

    for 0..99 {
        release(*stack, first);
        again := allocate(*stack, 100);
        require(again == first);
    }

    expect(owns(*stack, keep));
} @test

alignment_is_honoured :: () {
    storage: [8192] u8;
    stack: Stack_Allocator;

    init_in_memory(*stack, storage.data, storage.count);

    for alignment: s64.[1, 8, 16, 64, 256, 1024] {
        allocate(*stack, 3, 1);
        block := allocate(*stack, 10, alignment);
        require(block != null);
        expect_equal((cast(u64) block) % cast(u64) alignment, 0);
    }
} @test

full_stack_returns_null :: () {
    storage: [256] u8;
    stack: Stack_Allocator;

    init_in_memory(*stack, storage.data, storage.count);

    expect(allocate(*stack, 300) == null);
    expect(allocate(*stack, -1) == null);
    block := allocate(*stack, 150, 1);

    require(block != null);
    expect(allocate(*stack, 150, 1) == null);
    release(*stack, block);
    expect(allocate(*stack, 150, 1) == block); // room again once popped
} @test

out_of_order_release_is_deferred :: () {
    storage: [1024] u8;
    stack: Stack_Allocator;

    init_in_memory(*stack, storage.data, storage.count);

    a := allocate(*stack, 100);
    b := allocate(*stack, 100);
    c := allocate(*stack, 100);
    used := statistics(*stack).used;

    expect(!release(*stack, b)); // not the newest: nothing moves yet
    expect_equal(statistics(*stack).used, used);
    expect(release(*stack, c)); // pops c and the already-released b beneath it
    expect_equal(statistics(*stack).blocks, 1);
    expect(release(*stack, a));
    expect_equal(statistics(*stack).used, 0);
} @test

marks_and_rewind :: () {
    storage: [1024] u8;
    stack: Stack_Allocator;

    init_in_memory(*stack, storage.data, storage.count);

    keep := allocate(*stack, 100);
    outer := mark(*stack);

    allocate(*stack, 100);
    inner := mark(*stack);

    allocate(*stack, 100);

    rewind(*stack, inner);
    expect_equal(statistics(*stack).blocks, 2);
    rewind(*stack, outer);
    expect_equal(statistics(*stack).blocks, 1);
    expect(release(*stack, keep));

    allocate(*stack, 200);
    reset(*stack);
    expect_equal(statistics(*stack).used, 0);
    expect_equal(statistics(*stack).blocks, 0);
} @test

resizing :: () {
    storage: [2048] u8;
    stack: Stack_Allocator;

    init_in_memory(*stack, storage.data, storage.count);

    older := cast(*u8) allocate(*stack, 16);
    for 0..15 older[it] = cast(u8) it;

    newest := cast(*u8) allocate(*stack, 16);

    grown := resize(*stack, newest, 500); // newest: in place

    expect(grown == newest);
    expect(resize(*stack, newest, 100000) == null);
    expect(resize(*stack, newest, 8) == newest);

    moved := cast(*u8) resize(*stack, older, 300); // older: moves to the top

    require(moved != null);
    expect(moved != older);
    for 0..15 expect_equal(moved[it], cast(u8) it);

    expect(resize(*stack, null, 8) != null);
} @test

owned_block_comes_from_the_backing_allocator :: () {
    stack: Stack_Allocator;

    require(init(*stack, 4096));
    expect_not_null(allocate(*stack, 100));
    fini(*stack); // leak tracking checks the block went back
    expect(!init(*stack, -4));
} @test

works_as_context_allocator :: () {
    storage: [1 << 14] u8;
    stack: Stack_Allocator;

    init_in_memory(*stack, storage.data, storage.count);

    {
        push_allocator(as_allocator(*stack));
        text := copy_string("allocated on the stack");
        numbers: [..] int;

        for 0..99 array_add(*numbers, it);

        expect_equal(numbers[99], 99);
        expect_equal(text, "allocated on the stack");
        array_free(numbers);
        free(text);
    }

    caps := cast(Allocator_Caps) cast(u64) stack_allocator_proc(.CAPS, 0, 0, null, *stack);

    expect(caps & .FREE != 0 && caps & .IS_THIS_YOURS != 0);
    other: int;

    expect(stack_allocator_proc(.IS_THIS_YOURS, 0, 0, *other, *stack) == null);
} @test

// Wasm has no threads.
#if OS != .WASM {
    #import "Thread";

    Worker :: struct {
        stack: Stack_Allocator;
        storage: [4096] u8;
        wrong: s32;
    }

    each_thread_owns_a_stack :: () {
        workers: [4] Worker;
        threads: [4] Thread;

        for *threads {
            worker := *workers[it_index];

            init_in_memory(*worker.stack, worker.storage.data, worker.storage.count);
            it.data = worker;
            require(thread_init(it, (thread: *Thread) -> s64 {
                worker := cast(*Worker) thread.data;

                for 0..999 {
                    a := cast(*u8) allocate(*worker.stack, 200);
                    b := cast(*u8) allocate(*worker.stack, 300);

                    if !a || !b {
                        worker.wrong += 1;
                        continue;
                    }

                    memset(a, 0x11, 200);
                    memset(b, 0x22, 300);
                    if a[199] != 0x11 || b[299] != 0x22 worker.wrong += 1;

                    release(*worker.stack, b);
                    release(*worker.stack, a);
                }

                return 0;
            }));
            thread_start(it);
        }

        for *threads {
            expect(thread_is_done(it, -1));
            thread_deinit(it);
        }

        for workers {
            expect_equal(it.wrong, 0);
            expect_equal(statistics(*it.stack).used, 0);
        }
    } @test
}
