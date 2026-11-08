# Memory debugger

Basic's memory debugger records live allocations for leak reports and memory visualization. It runs during compile-time execution too when the program enables `MEMORY_DEBUGGER`.

## How it works

The allocation wrappers notify the debugger when an allocator returns storage or releases it. Records are indexed by address, with a separate index for heap handles. Allocator capabilities determine whether an individual free releases a tracked allocation.

Zero-byte allocation ranges are not recorded: a bump allocator can return the same address for several empty ranges and the next nonempty range. Recording those empty ranges would report a false duplicate live allocation. Heap handles remain tracked even though their records have size zero.

With visualization enabled, the debugger attempts to connect to a separate viewer. `Unable to connect to memory visualization channel` means that transport connection failed; allocation tracking still works.

## How to change it

Bookkeeping lives in `stdlib/internal/basic/memory-debugger.jai`; notifications originate in `allocator-call.jai` in the same directory. Keep heap identity separate from allocation size. The `Basic/tests/zero-size-pool-debugging.jai` regression checks empty strings, resizing an empty range and report totals with a flat-pool allocator.

```sh
JAIC_STDLIB="$PWD/stdlib" jacket test --path stdlib/Basic --exact zero-size-pool-debugging --interp
```

## Configuration

`#import "Basic"()(MEMORY_DEBUGGER = true, VISUALIZE_MEMORY_DEBUGGER = false)` enables allocation tracking without the viewer transport. These are Basic import parameters.

## Dependencies

The debugger uses Basic's context and allocator interface, `Hash_Table` for address indexes, stack-trace metadata and `Protocol_For_Memory_Visualization` for viewer messages.
