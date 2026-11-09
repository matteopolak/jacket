// A local's address is not a block of the allocator: freeing or resizing it is reported on every target instead of corrupting the heap.
#import "Extensions/Testing";
D :: #import "Default_Allocator";

freeing_a_local_is_reported :: () {
    local: [64] u8;

    D.allocator_proc(.FREE, 0, 64, local.data, null);
} @test @should_fail("Default_Allocator received an invalid pointer", "AddressSanitizer")

resizing_a_local_is_reported :: () {
    local: [64] u8;

    D.allocator_proc(.RESIZE, 128, 64, local.data, null);
} @test @should_fail("Default_Allocator received an invalid pointer", "AddressSanitizer")

freeing_a_local_in_a_callee_is_reported :: () {
    local: [32] u8;

    release_block(local.data);
} @test @should_fail("Default_Allocator received an invalid pointer", "AddressSanitizer")

release_block :: (block: *void) {
    padding: [256] u8;

    padding[0] = 1;
    D.allocator_proc(.FREE, 0, 32, block, null);
}

a_block_that_moved_is_still_accepted :: () {
    block := D.allocator_proc(.ALLOCATE, 60, 0, null, null);

    for size: 16..4096 {
        block = D.allocator_proc(.RESIZE, size * 4, (size - 1) * 4, block, null);
        expect(block != null);
    }

    D.allocator_proc(.FREE, 0, 4096 * 4, block, null);
} @test

freeing_a_block_twice_is_reported :: () {
    block := D.allocator_proc(.ALLOCATE, 24, 0, null, null);

    D.allocator_proc(.FREE, 0, 24, block, null);
    D.allocator_proc(.FREE, 0, 24, block, null);
} @test @should_fail("Default_Allocator received an invalid pointer", "AddressSanitizer")

// The Thread module has no wasm target.
#if OS != .WASM {
    T :: #import "Thread";

    blocks: [4] *void;

    blocks_freed_on_other_threads_are_accepted :: () {
        for *slot: blocks slot.* = D.allocator_proc(.ALLOCATE, 48, 0, null, null);

        workers: [4] T.Thread;
        for *worker: workers {
            worker.data = *blocks[it_index];
            expect(T.thread_init(worker, release_slot));
            T.thread_start(worker);
        }

        for *worker: workers {
            expect(T.thread_is_done(worker, 10000));
            T.thread_deinit(worker);
        }
    } @test

    release_slot :: (worker: *T.Thread) -> s64 {
        slot := cast(**void) worker.data;

        D.allocator_proc(.FREE, 0, 48, slot.*, null);

        return 0;
    }
}
