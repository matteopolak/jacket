# Tlsf

`#import "Extensions/Tlsf"` is a two-level segregated fit allocator in pure Jai: constant-time allocate and free over memory you give it, with a 16-byte header per block. Use it when allocation time must not vary, or to carve a fixed region (a static buffer, a file mapping) into freeable blocks.

```jai
#import "Extensions/Tlsf";

tlsf: Tlsf;
assert(init(*tlsf, 1024 * 1024));          // or init_in_memory(*tlsf, memory, bytes)
defer fini(*tlsf);

context.allocator = as_allocator(*tlsf);
numbers := NewArray(100, int);
array_free(numbers);
```

## How it works

- Free blocks are filed by size in a table of 32 first-level rows (powers of two from 256) with 16 second-level slots each; sizes under 256 bytes have an exact slot. Two bitmaps find the first non-empty slot at or above a request with two bit scans.
- Every block is `[previous, size|free, payload...]`. `previous` and the size give both physical neighbours, so `release` merges with free neighbours at once. A used block of size zero ends each pool, so no bounds checks are needed.
- A request rounds up to the next size class before the search so every block found is big enough. Near exhaustion a request can therefore fail while a free block up to 1/16 larger than it exists (never below 256 bytes).
- `resize` grows into a free next block, shrinks by returning the tail, and otherwise allocates, copies and releases; on failure the old block stays valid.
- `allocate_aligned` over-allocates, splits the leading gap off as a free block, and trims the tail. Any block, aligned or not, is freed with `release`.
- Allocation failure returns null; there are no asserts on exhaustion.
- `add_pool` gives a `Tlsf` up to 8 pools. `check` validates every block and table entry and is meant for tests; `statistics` walks the pools.

## Threads

A `Tlsf` has no lock by default. Created with `thread_safe = true`, `tlsf_proc` (and so `context.allocator`) takes a spin lock for each call and reports `MULTIPLE_THREADS`. The procedures `allocate`, `release` and `resize` never lock: call them from one thread or hold your own lock. Spin locks suit short critical sections, which these are, but a preempted lock holder stalls the others.

## How to change it

- Size classes: `SECOND_LEVEL_LOG2`, `SMALL_LIMIT`, `FIRST_LEVEL_SHIFT` and `FIRST_LEVELS` in `module.jai`. `FIRST_LEVELS` must stay 32 (a `u32` bitmap) and `MAX_BLOCK` follows from it (256 GiB).
- Header layout: `Block`. A free block's payload holds the free-list links, so `MIN_BLOCK` must stay at least 32.
- Any change should keep `check` passing in `tests/tlsf.jai`, which runs 20 000 random operations.

## Configuration

No flags or environment variables. `init(tlsf, bytes, thread_safe, backing)` takes the pool from `backing` (default `context.allocator`); `init_in_memory` takes memory you own.

## Dependencies

`Basic`, `Atomics` and `Bit_Operations`. `jacket bench --path stdlib/Extensions/Tlsf` compares it with the default allocator: a 100-allocation churn took 5.1 µs against 3.6 µs on an M-series Mac, with steady timing.
