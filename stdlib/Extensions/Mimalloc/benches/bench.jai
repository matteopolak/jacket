// Run with jacket bench --path stdlib/Extensions/Mimalloc. Each benchmark is 100 allocations and frees.
// test: only(.LINUX,.MACOS)
#import "Extensions/Testing";
#import "Extensions/Mimalloc";

SIZES :: s64.[16, 48, 100, 256, 700];

churn :: (b: *Bench, allocator: Allocator) {
    blocks: [100] *void;

    for b {
        for 0..99 blocks[it] = allocator.proc(.ALLOCATE, SIZES[it % 5], 0, null, allocator.data);
        for 0..99 allocator.proc(.FREE, 0, 0, blocks[it], allocator.data);
        black_box(blocks[0]);
    }
}

default_allocator_churn :: (b: *Bench) {
    churn(b, context.allocator);
} @bench

mimalloc_churn :: (b: *Bench) {
    churn(b, mimalloc_allocator);
} @bench
