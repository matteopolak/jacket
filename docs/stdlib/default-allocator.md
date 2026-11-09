# Default allocator

`Default_Allocator` is the allocator behind `context.allocator`. Two backends share one module: a heap allocator on x64 (except iOS), and a thin wrapper over the C library's `malloc` everywhere else (arm64 macOS and Linux, wasm, iOS).

## How it works

- x64: every live block is in a table keyed by address (`default-heaps.jai`). A free or resize of a pointer the table does not hold is reported.
- Everywhere else (`default-c.jai`): each block starts with a 16-byte prefix. The first word is a tag, the address of the contents mixed with a constant; the second is the region word that `aligned-blocks.jai` needs for `free`. A free or resize checks the tag first. A local, a global, memory from another allocator or a block that was already freed has no tag, so the call is refused: `Default_Allocator received an invalid pointer or source extent`, then null. Nothing reaches `free`.
- The report is an `assert`, so it can be turned off with the module parameter `ENABLE_ASSERTS = false`; the pointer is still left alone.

The check catches any non-owned pointer, not only stack addresses, and costs one load and compare per call. It reads the word 16 bytes before the pointer, which the allocator already read for the region word.

## How to change it

- The tag is written by `tagged` after every `ALLOCATE` and `RESIZE`, because a resize can move the contents (`regrow_contents` does not move the prefix with them). Anything new that hands out or moves a block must call it.
- `untag` runs before the block is released, which is what makes a second free of it fail.
- The prefix is 16 bytes (`PREFIX`) to keep `malloc` alignment; the tag takes the half the region word does not use.

## Configuration

- `ENABLE_ASSERTS` (default `true`): report invalid pointers with `assert`.
- `ENABLE_VALIDATE_ARGS` (default `true`): check sizes on the x64 backend.

## Dependencies

`internal/libc` for `malloc`, `realloc` and `free`; `Basic` for `assert`.

## Tests

`stdlib/Default_Allocator/tests/stack-addresses.jai` frees and resizes locals, frees a block twice, and frees blocks on other threads. It runs natively and under `--interp`, on every target.
