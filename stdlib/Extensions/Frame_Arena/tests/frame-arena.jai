// Frame and double-frame arenas: allocate, resize, alignment, reuse after reset, frame lifetimes, and threads.
#import "Basic";
#import "Extensions/Testing";
#import "Extensions/Frame_Arena";

allocates_in_order :: () {
    storage: [1024] u8;
    arena: Frame_Arena;

    init_in_memory(*arena, storage.data, storage.count);

    a := cast(*u8) allocate(*arena, 100);
    b := cast(*u8) allocate(*arena, 100);

    require(a != null && b != null);
    expect(b >= a + 100);
    expect(owns(*arena, a) && owns(*arena, b));
    expect(!owns(*arena, storage.data + 1000));

    memset(a, 1, 100);
    memset(b, 2, 100);

    for 0..99 {
        expect_equal(a[it], 1);
        expect_equal(b[it], 2);
    }
} @test

alignment_is_honoured :: () {
    storage: [8192] u8;
    arena: Frame_Arena;

    init_in_memory(*arena, storage.data, storage.count);

    for alignment: s64.[1, 2, 8, 16, 64, 256, 1024] {
        allocate(*arena, 3, 1); // knock the cursor off alignment
        block := allocate(*arena, 10, alignment);
        require(block != null);
        expect_equal((cast(u64) block) % cast(u64) alignment, 0);
    }
} @test

full_arena_returns_null :: () {
    storage: [256] u8;
    arena: Frame_Arena;

    init_in_memory(*arena, storage.data, storage.count);

    expect(allocate(*arena, 257) == null);
    expect(allocate(*arena, -1) == null);
    expect(allocate(*arena, 200, 1) != null);
    expect(allocate(*arena, 100, 1) == null);
    expect(allocate(*arena, 56, 1) != null); // exactly the rest
    expect(allocate(*arena, 1, 1) == null);
    expect(allocate(*arena, 0, 1) != null); // zero bytes always fit
} @test

reset_reuses_the_memory :: () {
    storage: [512] u8;
    arena: Frame_Arena;

    init_in_memory(*arena, storage.data, storage.count);

    start: *void;

    for frame: 0..4 {
        first := allocate(*arena, 200);

        require(first != null);
        if frame == 0 start = first;

        expect(first == start);
        expect(allocate(*arena, 200) != null);
        expect(allocate(*arena, 200) == null);
        reset(*arena);
    }

    stats := statistics(*arena);

    expect_equal(stats.frame, 5);
    expect_equal(stats.used, 0);
    expect(stats.high_water >= 400 && stats.high_water <= 512);
} @test

reset_can_poison :: () {
    storage: [128] u8;
    arena: Frame_Arena;

    init_in_memory(*arena, storage.data, storage.count);
    arena.poison_on_reset = true;

    block := cast(*u8) allocate(*arena, 32);

    memset(block, 7, 32);
    reset(*arena);
    for 0..31 expect_equal(block[it], 0xcc);
} @test

resizing :: () {
    storage: [1024] u8;
    arena: Frame_Arena;

    init_in_memory(*arena, storage.data, storage.count);

    first := cast(*u8) allocate(*arena, 16);
    for 0..15 first[it] = cast(u8) it;

    grown := cast(*u8) resize(*arena, first, 16, 400); // the latest allocation grows in place

    expect(grown == first);
    expect(resize(*arena, first, 400, 2000) == null); // no room: the block stays valid
    expect_equal(first[15], 15);

    second := allocate(*arena, 32);

    require(second != null);
    moved := cast(*u8) resize(*arena, first, 16, 100); // no longer the latest: copied

    require(moved != null);
    expect(moved != first);
    for 0..15 expect_equal(moved[it], cast(u8) it);

    expect(resize(*arena, null, 0, 8) != null);
} @test

owned_block_comes_from_the_backing_allocator :: () {
    arena: Frame_Arena;

    require(init(*arena, 4096));
    expect_not_null(allocate(*arena, 100));
    fini(*arena); // leak tracking checks the block went back
    expect(!init(*arena, 0));
} @test

works_as_context_allocator :: () {
    storage: [1 << 14] u8;
    arena: Frame_Arena;

    init_in_memory(*arena, storage.data, storage.count);

    {
        push_allocator(as_allocator(*arena));
        numbers: [..] int;
        for 0..99 array_add(*numbers, it);

        expect_equal(numbers[99], 99);
        free(numbers.data); // free does nothing
    }

    expect(statistics(*arena).used > 0);
    caps := cast(Allocator_Caps) cast(u64) frame_arena_proc(.CAPS, 0, 0, null, *arena);

    expect(caps & .HINT_I_AM_PER_FRAME_TEMPORARY_STORAGE != 0);
    block := frame_arena_proc(.ALLOCATE, 8, 0, null, *arena);
    other: int;

    expect(frame_arena_proc(.IS_THIS_YOURS, 0, 0, block, *arena) != null);
    expect(frame_arena_proc(.IS_THIS_YOURS, 0, 0, *other, *arena) == null);
    expect(frame_arena_proc(.IS_THIS_YOURS, 0, 0, null, *arena) == null);
} @test

double_frame_keeps_the_previous_frame :: () {
    frames: Double_Frame_Arena;

    require(init(*frames, 1024));
    defer fini(*frames);

    one := cast(*s64) allocate(*frames, 8);
    one.* = 111;
    expect(owns(current(*frames), one));

    next_frame(*frames);
    two := cast(*s64) allocate(*frames, 8);
    two.* = 222;
    expect(one != two);
    expect_equal(one.*, 111); // the previous frame is still intact
    expect(owns(previous(*frames), one));
    expect(owns(*frames, one) && owns(*frames, two));

    next_frame(*frames);
    three := cast(*s64) allocate(*frames, 8);

    expect(three == one); // the first frame's memory came back
    expect_equal(two.*, 222);
    expect_equal(statistics(previous(*frames)).frame, 1);
} @test

double_frame_as_allocator :: () {
    frames: Double_Frame_Arena;

    require(init(*frames, 4096));
    defer fini(*frames);
    context.allocator = as_allocator(*frames);

    text := copy_string("from frame one");

    next_frame(*frames);
    other := copy_string("from frame two");

    expect_equal(text, "from frame one");
    expect_equal(other, "from frame two");

    // Resizing a previous-frame block copies it into the current frame.
    bigger := cast(*u8) double_frame_arena_proc(.RESIZE, 64, text.count, text.data, *frames);

    require(bigger != null);
    expect(owns(current(*frames), bigger));
    expect_equal(string.{ text.count, bigger }, "from frame one");

    caps := cast(Allocator_Caps) cast(u64) double_frame_arena_proc(.CAPS, 0, 0, null, *frames);

    expect(caps & .HINT_I_AM_PER_FRAME_TEMPORARY_STORAGE != 0);
} @test

Worker :: struct {
    arena: Frame_Arena;
    storage: [4096] u8;
    wrong: s32;
}

each_thread_owns_an_arena :: () {
    #import "Thread";
    workers: [4] Worker;
    threads: [4] Thread;

    for *threads {
        worker := *workers[it_index];

        init_in_memory(*worker.arena, worker.storage.data, worker.storage.count);
        it.data = worker;
        require(thread_init(it, (thread: *Thread) -> s64 {
            worker := cast(*Worker) thread.data;

            for frame: 0..99 {
                for 0..9 {
                    block := cast(*u8) allocate(*worker.arena, 100);
                    if !block {
                        worker.wrong += 1;
                        continue;
                    }

                    memset(block, 0x5a, 100);
                    for 0..99 if block[it] != 0x5a worker.wrong += 1;
                }

                reset(*worker.arena);
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
        expect_equal(statistics(*it.arena).frame, 100);
    }
} @test @only(.MACOS,.LINUX,.WINDOWS)
