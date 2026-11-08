# Memory debugger

Basic's memory debugger records live allocations for leak reports and memory visualization. It runs during compile-time execution too when the program enables `MEMORY_DEBUGGER`.

## How it works

The allocation wrappers tell the debugger when an allocator hands out or releases storage. Records are indexed by address, with a separate index for heap handles. Allocator capabilities decide whether an individual free releases a tracked allocation.

Zero-byte ranges are not recorded: a bump allocator can return one address for several empty ranges and the next nonempty one, which would look like a duplicate live allocation. Heap handles stay tracked even though their records have size zero.

With visualization on, the debugger tries to connect to a separate viewer. `Unable to connect to memory visualization channel` means that connection failed; tracking still works.

## How to change it

Bookkeeping is in `stdlib/internal/basic/memory-debugger.jai`; notifications come from `allocator-call.jai` beside it. Keep heap identity separate from allocation size. `Basic/tests/zero-size-pool-debugging.jai` covers empty strings, resizing an empty range and report totals with a flat-pool allocator.

```sh
JAIC_STDLIB="$PWD/stdlib" jacket test --path stdlib/Basic --exact zero-size-pool-debugging --interp
```

## Configuration

`#import "Basic"()(MEMORY_DEBUGGER = true, VISUALIZE_MEMORY_DEBUGGER = false)` enables tracking without the viewer. Both are Basic import parameters.

## Dependencies

The debugger uses Basic's context and allocator interface, `Hash_Table` for address indexes, stack-trace metadata and `Protocol_For_Memory_Visualization` for viewer messages.
