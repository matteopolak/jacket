# Memory debugger

Basic's memory debugger records live allocations for leak reports and memory visualization. It runs during compile-time execution too when the program enables `MEMORY_DEBUGGER`.

## How it works

The allocation wrappers tell the debugger when an allocator hands out or releases storage. Records are indexed by address, with a separate index for heap handles. Allocator capabilities decide whether an individual free releases a tracked allocation.

An allocator that ignores `.FREE` and gives everything back at once (`Pool`, `Flat_Pool`) reports that with `release_all_allocations(Allocator.{ proc, data })` from its reset and fini. Every live range the allocator owns stops being live, and ranges handed out afterwards are tracked as new. The call costs the number of ranges that allocator owns: the debugger keeps a list per owner, keyed by the allocator's data pointer, and `destroy_heap` uses the same lists. The allocator must have instance data. It reaches the debugger through a hook in `internal/basic-context`, so it works whichever `Basic` import parameters the allocator's own module used, and does nothing when no debugger is tracking.

Zero-byte ranges are not recorded: a bump allocator can return one address for several empty ranges and the next nonempty one, which would look like a duplicate live allocation. Heap handles stay tracked even though their records have size zero.

With visualization on, the debugger tries to connect to a separate viewer. `Unable to connect to memory visualization channel` means that connection failed; tracking still works.

## How to change it

Bookkeeping is in `stdlib/internal/basic/memory-debugger.jai`; notifications come from `allocator-call.jai` beside it. Keep heap identity separate from allocation size. `Basic/tests/zero-size-pool-debugging.jai` covers empty strings, resizing an empty range and report totals with a flat-pool allocator; `flat-pool-debugging.jai` and `pool-debugging.jai` cover reset and fini.

```sh
JAIC_STDLIB="$PWD/stdlib" jacket test --path stdlib/Basic --exact zero-size-pool-debugging --interp
```

## Configuration

`#import "Basic"()(MEMORY_DEBUGGER = true, VISUALIZE_MEMORY_DEBUGGER = false)` enables tracking without the viewer. Both are Basic import parameters.

## Dependencies

The debugger uses Basic's context and allocator interface, `Hash_Table` for address indexes, stack-trace metadata and `Protocol_For_Memory_Visualization` for viewer messages.
