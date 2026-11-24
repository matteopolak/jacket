// Run with jacket bench --path stdlib/Extensions/Stack_Allocator. Each benchmark is 100 allocations and frees.
#import "Extensions/Testing";
#import "Basic";
#import "Extensions/Stack_Allocator";

SIZES :: s64.[16, 48, 100, 256, 700];

default_allocator_stack :: (b: *Bench) {
    blocks: [100] *void;

    for b {
        for 0..99 blocks[it] = alloc(SIZES[it % 5]);
        for < 99..0 free(blocks[it]);
        black_box(blocks[0]);
    }
} @bench

stack_allocator_stack :: (b: *Bench) {
    stack: Stack_Allocator;

    assert(init(*stack, 1 << 16));
    defer fini(*stack);
    blocks: [100] *void;

    for b {
        for 0..99 blocks[it] = allocate(*stack, SIZES[it % 5]);
        for < 99..0 release(*stack, blocks[it]);
        black_box(blocks[0]);
    }
} @bench
