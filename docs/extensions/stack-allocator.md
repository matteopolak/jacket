# Stack_Allocator

`#import "Extensions/Stack_Allocator"` is a last-in, first-out allocator over a fixed block. Scoped scratch data costs a pointer move; `mark` and `rewind` free a whole scope at once.

```jai
#import "Extensions/Stack_Allocator";

stack: Stack_Allocator;
assert(init(*stack, 64 * 1024));
defer fini(*stack);

scope := mark(*stack);
scratch := allocate(*stack, 1000);
rewind(*stack, scope);
```

## How it works

- Each block has a 24-byte header just before its payload (previous cursor, previous newest block, size). `allocate` aligns the payload (16 by default), writes the header and advances the cursor. A full stack returns null.
- `release` of the newest block restores the cursor from its header, then keeps popping blocks that were released earlier out of order.
- `release` of any other block marks it dead and changes nothing else. Its space comes back when the blocks above it pop or a `rewind`/`reset` covers it. It returns whether the cursor moved. Through `context.allocator` this means generic code can free in any order without crashing; only LIFO frees give memory back immediately.
- `resize` of the newest block is in place. An older block moves to the top and its old space follows the rule above.
- `mark` records the cursor; `rewind(mark)` frees everything after it, released or not. Rewind in reverse order of marking.
- `statistics` reports use (headers and padding included), high-water mark and live blocks.
- Not thread safe: give each thread its own stack.

## How to change it

- Header fields: `Header` and `HEADER_SIZE` (keep `HEADER_SIZE` equal to the struct size, a multiple of 8).
- Making out-of-order release an error: assert in `release` where it sets `RELEASED`.

## Configuration

`init(stack, capacity, backing)` takes the block from `backing` (default `context.allocator`); `init_in_memory` takes memory you own.

## Dependencies

`Basic`. `jacket bench --path stdlib/Extensions/Stack_Allocator`: 100 allocations and reverse-order frees took 0.15 µs against 4.6 µs for the default allocator.
