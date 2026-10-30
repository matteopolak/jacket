# Pool


## Public declaration inventory

Only names visible through the module’s exported scope are listed. Declarations from conditional branches retain their condition label; implementation-local declarations are omitted. Aggregate fields and enum members are recorded in the dedicated type-shape section.

- `POOL_BUCKET_SIZE_DEFAULT :: 65536`
- `POOL_OVERSIZED_SIZE_DEFAULT :: 6554`
- `STAMP :: 0xcc`
- `HEADER_SIZE :: size_of(Memblock)`
- `Pool :: struct`
- `set_allocators :: (pool: *Pool, block_allocator : Allocator = .{})`
- `get :: (using pool: *Pool, nbytes: s64) -> *void`
- `release :: (using pool: *Pool)`
- `reset :: (using pool: *Pool)`
- `pool_allocator_proc :: (mode: Allocator_Mode, requested_size: s64, old_size: s64, old_memory: *void, allocator_data: *void) -> *void`
- `calculate_used_memory :: (using pool: Pool) -> inband_size: s64, outband_count: s64`


- `example_usage :: ()`

## Public aggregate shapes and enum values

Each block gives the exported type declaration with its member fields or enum values. Conditional declarations with distinct shapes are shown as separate variants; platform labels describe availability, not source organization.

### `Pool`

```jai
Pool :: struct {

    memblock_size:   s64 = POOL_BUCKET_SIZE_DEFAULT;
    oversized_size:  s64 = POOL_OVERSIZED_SIZE_DEFAULT;
    alignment := 8;

    current_memblock : *Memblock;
    current_pos      : *u8;
    bytes_left       : s64;

    block_allocator  : Allocator;

    unused_memblocks:        *Memblock;
    used_memblocks:          *Memblock;
    first_used_memblock:     *Memblock;
    out_of_band_allocations: *Memblock;

    overwrite_memory        := false;

    free_memblocks_on_reset := false;

    logging_options: enum_flags u32 {
        LOG_OVERSIZED_ALLOCATIONS             :: 0x1;
        LOG_OVERSIZED_ALLOCATIONS_AS_ERRORS   :: 0x2;
        LOG_OVERSIZED_ALLOCATIONS_STACK_TRACE :: 0x4;
    }
}
```

The pointer fields `current_memblock`, `unused_memblocks`, `used_memblocks`, `first_used_memblock`, and `out_of_band_allocations` expose the same private linked-header shape when callers inspect them: each node has `next: *same node` and a `data: void` flexible tail marker. This is a reachable layout through public fields, but the helper type itself is not an exported module name; clients should treat these links as allocator bookkeeping rather than a supported mutation interface.
## Behavioral contract

`Pool` is a bulk-lifetime allocator. `get(nbytes)` returns uninitialized storage aligned to `alignment` (default 8), advancing within a block; negative sizes are invalid. If `nbytes >= oversized_size`, it obtains a separately tracked out-of-band allocation from `block_allocator`. If the backing allocator fails, `get` returns null. `pool_allocator_proc` answers `IS_THIS_YOURS` for addresses inside any block it handed out, ordinary or oversized. Ordinary block exhaustion requests another block; failure to obtain one also returns null. Configure the backing allocator before allocations that depend on it. `pool_allocator_proc` selects the default allocator on its first allocation when `block_allocator.proc` is null.

`reset` invalidates all returned storage, frees all out-of-band allocations, and either retains ordinary blocks for reuse or frees them when `free_memblocks_on_reset` is true. With `overwrite_memory`, used blocks are stamped to help expose stale references. `release` performs reset and then frees retained blocks as well. `pool_allocator_proc` treats individual FREE as a no-op; RESIZE allocates fresh pool storage and copies the common prefix, so the old allocation remains part of the pool until bulk cleanup. It rejects thread start/stop and heap create/destroy operations by assertion, and ownership query only recognizes addresses within tracked used blocks. `calculate_used_memory` reports in-band bytes accounted from used blocks minus remaining bytes in the current block, plus the count of outstanding out-of-band blocks; it is accounting, not a per-allocation size query.

The default block and oversized thresholds are 65,536 and 6,554 bytes; alignment defaults to 8 and must be a power of two (it is not validated). Optional overwrite-on-reset stamps used blocks to help expose stale pointers. The module parameter `USE_UNMAPPING_ALLOCATOR` defaults false. When true, and no explicit allocator is passed to `set_allocators`, the pool selects the Unmapping_Allocator and sets `free_memblocks_on_reset` true. An explicitly supplied allocator still takes precedence and does not have that flag changed by this option. Consequently reset frees both ordinary and oversized backing allocations through the selected allocator; release additionally frees any retained ordinary blocks. The pool does not itself own or destroy the selected allocator. Any retention or page-protection behavior after a backing `FREE` is that allocator's contract (the Unmapping_Allocator retains freed mappings as inaccessible memory). With the option false and no explicit allocator, the active context allocator is captured.

`set_allocators` checks only whether `pool.block_allocator.proc` is non-null. If it is non-null, the call logs an error and leaves the existing allocator selection unchanged. This guard is based on the current allocator field, not a separate historical-use marker or whether blocks remain allocated. Thus a default-initialized pool can be configured; after the backing allocator has been selected, another configuration call is rejected even before the first allocation. A normal block allocation through `get` or the allocator procedure selects the backing allocator if needed, so later configuration is rejected. `reset` clears current-block and allocation-list state but preserves `block_allocator`; it also preserves `free_memblocks_on_reset`. This holds whether reset retains ordinary blocks or frees them. `release` first resets and then frees retained blocks, but also preserves the allocator descriptor and option fields. Therefore `set_allocators` remains rejected after either kind of reset or after `release`. The module defines no separate `fini` operation. This is the source's observable configuration guard, not a claim that copying a `Pool` value is forbidden. For safe use, select one backing allocator before allocations and keep its lifetime valid for every retained pool block.

Depends on the supplied/context allocator and, when enabled, the unmapping allocator; oversized-allocation logging can use Basic logging and stack traces.

## Portability notes

The `USE_UNMAPPING_ALLOCATOR` module parameter is an extension of this repository: the official `Pool` module takes different parameters, so `#import "Pool"(USE_UNMAPPING_ALLOCATOR = true)` is `Too many module parameters` there. Code that must build with both compilers imports `Pool` plainly.
