# Default allocator

`Default_Allocator` is the allocator behind `context.allocator`. Two backends share one module: a heap allocator on x64 (except iOS), and a thin wrapper over the C library's `malloc` everywhere else (arm64 macOS and Linux, wasm, iOS).

## How it works

- x64: every live block is in a table keyed by address (`default-heaps.jai`). A free or resize of a pointer the table does not hold is reported.
- Everywhere else (`default-c.jai`): each block starts with a 32-byte prefix. Its words are the size the block was last allocated or resized to, a tag (the address of the contents mixed with a constant) and the region word that `aligned-blocks.jai` needs for `free`. A free or resize checks the tag first, then that its `old_size` is not larger than the recorded size. A local, a global, memory from another allocator or a block that was already freed has no tag, and a claim larger than the block fails the size check, so the call is refused: `Default_Allocator received an invalid pointer or source extent`, then null. Nothing reaches `free`.
- The report is an `assert`, so it can be turned off with the module parameter `ENABLE_ASSERTS = false`; the pointer is still left alone.

The check catches any non-owned pointer, not only stack addresses, and costs one load and compare per call. It reads the words just before the pointer, which share a cache line with the region word the allocator already read.

## How to change it

- The tag is written by `tagged` (which also records the size) after every `ALLOCATE` and `RESIZE`, because a resize can move the contents (`regrow_contents` does not move the prefix with them). Anything new that hands out or moves a block must call it.
- `untag` runs before the block is released, which is what makes a second free of it fail.
- The prefix is 32 bytes (`PREFIX`) to keep `malloc` alignment; the region word is last, the tag before it, the size before that.

## Configuration

- `ENABLE_ASSERTS` (default `true`): report invalid pointers with `assert`.
- `ENABLE_VALIDATE_ARGS` (default `true`): check sizes on the x64 backend.

## Dependencies

`internal/libc` for `malloc`, `realloc` and `free`; `Basic` for `assert`.

## Tests

`stdlib/Default_Allocator/tests/stack-addresses.jai` frees and resizes locals, frees a block twice, and frees blocks on other threads. It runs natively and under `--interp`, on every target.
