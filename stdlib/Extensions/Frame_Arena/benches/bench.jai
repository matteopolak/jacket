// Run with jacket bench --path stdlib/Extensions/Frame_Arena. Each benchmark is one frame of 100 allocations.
#import "Extensions/Testing";
#import "Basic";
#import "Extensions/Frame_Arena";

SIZES :: s64.[16, 48, 100, 256, 700];

default_allocator_frame :: (b: *Bench) {
    blocks: [100] *void;

    for b {
        for 0..99 blocks[it] = alloc(SIZES[it % 5]);
        for 0..99 free(blocks[it]);
        black_box(blocks[0]);
    }
} @bench

frame_arena_frame :: (b: *Bench) {
    arena: Frame_Arena;

    assert(init(*arena, 1 << 16));
    defer fini(*arena);
    last: *void;

    for b {
        for 0..99 last = allocate(*arena, SIZES[it % 5]);
        reset(*arena);
        black_box(last);
    }
} @bench

double_frame_arena_frame :: (b: *Bench) {
    frames: Double_Frame_Arena;

    assert(init(*frames, 1 << 16));
    defer fini(*frames);
    last: *void;

    for b {
        for 0..99 last = allocate(*frames, SIZES[it % 5]);
        next_frame(*frames);
        black_box(last);
    }
} @bench
