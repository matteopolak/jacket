// test: only(.LINUX,.MACOS)
// mimalloc as `context.allocator`: allocate, resize, free, alignment, reuse, separate heaps and threads.
#if OS == .MACOS || OS == .LINUX {
    #import "Basic";
    #import "Extensions/Testing";
    #import "Extensions/Mimalloc";
    Thread :: #import "Thread";

    allocates_resizes_and_frees :: () {
        context.allocator = mimalloc_allocator;
        numbers := NewArray(1000, s64);
        for *numbers it.* = it_index * 3;

        expect_equal(numbers.count, 1000);

        grown := cast(*s64) realloc(numbers.data, 8000 * 8, 1000 * 8);

        expect_not_null(grown);
        for 0..999 expect_equal(grown[it], it * 3);

        free(grown);

        block := alloc(100);

        expect(usable_size(block) >= 100);
        expect(mimalloc_proc(.IS_THIS_YOURS, 0, 0, block, null) != null);
        free(block);
    } @test

    aligned_blocks :: () {
        for alignment: s64.[16, 64, 4096, 65536] {
            block := allocate_aligned(100, alignment);
            expect_not_null(block);
            expect_equal((cast(u64) block) % cast(u64) alignment, 0);
            mimalloc_proc(.FREE, 0, 0, block, null);
        }
    } @test

    freed_blocks_are_reused :: () {
        first: [64] *void;
        for *first it.* = mimalloc_proc(.ALLOCATE, 48, 0, null, null);
        for first mimalloc_proc(.FREE, 0, 0, it, null);

        // The freed blocks come back instead of the heap growing.
        for 0..63 {
            block := mimalloc_proc(.ALLOCATE, 48, 0, null, null);
            found := false;

            for first if it == block found = true;

            expect(found);
            mimalloc_proc(.FREE, 0, 0, block, null);
        }
    } @test

    separate_heaps :: () {
        heap := make_heap();
        block := alloc(64,, heap);

        expect_not_null(block);
        expect(mimalloc_proc(.IS_THIS_YOURS, 0, 0, block, null) != null);
        grown := realloc(block, 4096, 64,, heap);

        expect_not_null(grown);
        release_heap(heap);
    } @test

    caps_report :: () {
        caps := cast(Allocator_Caps) cast(u64) mimalloc_proc(.CAPS, 0, 0, null, null);
        expect(caps & .MULTIPLE_THREADS != 0);
        expect(caps & .CREATE_HEAP != 0);
        expect(caps & .FREE != 0);
    } @test

    Shared :: struct {
        freed_on_other_thread: *void;
    }

    other_thread_frees :: () {
        shared: Shared;
        shared.freed_on_other_thread = mimalloc_proc(.ALLOCATE, 256, 0, null, null);

        thread: Thread.Thread;
        thread.data = *shared;
        expect(Thread.thread_init(*thread, (t: *Thread.Thread) -> s64 {
            shared := cast(*Shared) t.data;

            for 0..999 {
                block := mimalloc_proc(.ALLOCATE, 32 + it, 0, null, null);
                mimalloc_proc(.FREE, 0, 0, block, null);
            }

            mimalloc_proc(.FREE, 0, 0, shared.freed_on_other_thread, null);

            return 0;
        }));

        Thread.thread_start(*thread);
        expect(Thread.thread_is_done(*thread, -1));
        Thread.thread_deinit(*thread);
    } @test
}
