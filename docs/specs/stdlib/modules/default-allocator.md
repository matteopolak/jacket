# Default_Allocator

## Public declaration inventory

The allocator interface and entry points are exported. The module parameters are `ENABLE_ASSERTS := true` and `ENABLE_VALIDATE_ARGS := true`. The internal size-class, span, system-call, and statistics-update helpers are module implementation details and are not listed as public declarations.

X64 (except iOS) and Nintendo Switch 2:

- `allocator :: Allocator.{allocator_proc, null}`
- `allocator_default_heap: *Heap`
- `allocator_proc :: (mode: Allocator_Mode, requested_size: s64, old_size: s64, old_memory: *void, allocator_data: *void) -> *void`
- `initialize_if_needed :: inline () -> did_init: bool`
- `finalize :: ()`
- `unsafe_reset :: inline ()`
- `is_initialized :: inline () -> bool`
- `usable_size :: inline (ptr: *void) -> s64`
- `is_this_yours :: (heap: *Heap, memory: *void) -> bool`
- `heap_free_all :: (heap: *Heap)`
- `global_statistics :: () -> Global_Statistics`
- `heap_statistics :: (heap: *Heap) -> Heap_Statistics`

Fallback targets:

- `allocator_proc :: (mode: Allocator_Mode, size: s64, old_size: s64, old_memory: *void, allocator_data: *void) -> *void`
- `unsafe_reset :: inline ()`

## Public type members

#### `SYSTEM_INFO` (struct; Windows)

- `_pad0: u32`
- `dwPageSize: u32`
- `_pad1: [8]u32`
- `dwAllocationGranularity: u32`
- `_pad2: u32`

#### `Span` (struct)

- `free_list: *void`
- `block_count: u32`
- `size_class: u32`
- `free_list_limit: u32`
- `used_count: u32`
- `free_list_deferred: *void`
- `list_size: u32`
- `block_size: u32`
- `flags: enum_flags u32` with `MASTER = 1`, `SUBSPAN = 2`, `ALIGNED_BLOCKS = 4`, and `UNMAPPED_MASTER = 8`
- `span_count: u32`
- `total_spans: u32`
- `offset_from_master: u32`
- `remaining_spans: s32`
- `align_offset: u32`
- `heap: *Heap`
- `next: *Span`
- `prev: *Span`

#### `Heap` (struct; X64 implementation)

- `lock: *void`
- `is_orphaned: bool`
- `size_class: [126]Heap_Size_Class`
- `span_free_deferred: *void`
- `full_span_count: s64`
- `span_reserve: *Span`
- `span_reserve_master: *Span`
- `spans_reserved: u32`
- `child_count: s32`
- `next_heap: *Heap`
- `next_orphan: *Heap`
- `id: s32`
- `finalize: s32`
- `master_heap: *Heap`
- `full_span: [126]*Span`
- `large_huge_span: *Span`
- With `ENABLE_STATISTICS` in the statistics-enabled variant: `lock` is `s32` instead of `*void`; `span_use: [63]Span_Use`, `size_class_use: [127]Size_Class_Use`, `thread_to_global: s64`, and `global_to_thread: s64` are added.

#### `Heap_Size_Class` (struct)

- `free_list: *void`
- `partial_span: *Span`
- `cache: *Span`

#### `Size_Class` (struct)

- `block_size: u32`
- `block_count: u16`
- `class_idx: u16`

#### `Global_Statistics` (struct)

The standard implementation returns this aggregate with the following fields; its statistics are documented as dummy/zero-valued in that implementation:

- `mapped: s64`
- `mapped_peak: s64`
- `cached: s64`
- `huge_alloc: s64`
- `huge_alloc_peak: s64`
- `mapped_total: s64`
- `unmapped_total: s64`

In the statistics-enabled variant, the same field names and types are present, but the counters report mapped memory, peak mapped memory, cached memory, current/peak huge allocation, and cumulative mapped/unmapped memory. This variant has conditional counter semantics controlled by `ENABLE_STATISTICS`.

#### `Heap_Statistics` (struct)

Standard implementation:

- `sizecache: s64`
- `spancache: s64`

Statistics-enabled variant adds:

- `thread_to_global: s64`
- `global_to_thread: s64`
- `span_use: [64]struct { current: s64; peak: s64; to_global: s64; from_global: s64; to_cache: s64; from_cache: s64; to_reserved: s64; from_reserved: s64; map_calls: s64; }`
- `size_use: [128]struct { alloc_current: s64; alloc_peak: s64; alloc_total: s64; free_total: s64; spans_to_cache: s64; spans_from_cache: s64; spans_from_reserved: s64; map_calls: s64; }`

#### `Span_Use` (statistics-enabled implementation type)

- `current: s32`
- `high: s32`
- `spans_deferred: s32`
- `spans_to_global: s32`
- `spans_from_global: s32`
- `spans_to_cache: s32`
- `spans_from_cache: s32`
- `spans_to_reserved: s32`
- `spans_from_reserved: s32`
- `spans_map_calls: s32`

#### `Size_Class_Use` (statistics-enabled implementation type)

- `alloc_current: s32`
- `alloc_peak: s32`
- `alloc_total: s32`
- `free_total: s32`
- `spans_current: s32`
- `spans_peak: s32`
- `spans_to_cache: s32`
- `spans_from_cache: s32`
- `spans_from_reserved: s32`
- `spans_map_calls: s32`
- `unused: s32`

## Behavioral contract and constraints

The optimized allocator branch is selected for X64 except iOS, and for Nintendo Switch 2. Other targets use the C runtime allocation fallback. The optimized branch advertises heap creation, freeing, resizing, and ownership checks through `Allocator_Caps`; the fallback does not support heap creation or ownership checks and advertises multiple-thread use, freeing, and resizing. The capability set has no separate multiple-heaps bit: `CREATE_HEAP` indicates support for creating additional heap handles. `allocator_proc` uses `allocator_data` as an optional heap handle in the optimized branch. Its private size-class configuration has 65 small classes at 16-byte granularity, 61 medium classes at 512-byte granularity, and 63 large span classes; these are implementation parameters, not exported constants. With `ENABLE_VALIDATE_ARGS` (default true), optimized allocation and resize reject sizes at or above the largest signed 64-bit value minus one memory span. The `ENABLE_STATISTICS` branch is present only in the statistics-enabled variant; that variant changes the `Heap`/statistics shapes and counter semantics, while the standard optimized branch has no live statistics counters. Non-optimized fallback targets do not expose the optimized heap/statistics API.

With argument validation enabled, optimized allocation and resize requests at or above the limit described above return null. Allocation failure otherwise returns null; no stronger failure guarantee is documented. Pointers must be released with the compatible allocator and are invalid after release. `unsafe_reset` is intended only in a child process after `fork`, to clear inherited lock state; its fallback implementation is a no-op.

Both branches start a block on a 64-byte boundary when it is 64 bytes or larger, and otherwise keep malloc's alignment, so `New` of a type declared `#align 64` (whose size is a multiple of 64) is aligned and freed with `free` like any other block. Each block is carved out of one C allocation with the C pointer stored in the word before the contents (`stdlib/internal/allocation/aligned-blocks.jai`); only malloc, realloc, memmove and free are used so that compile-time code of a cross build works on every host. A block that is resized to 64 bytes or more is moved onto a boundary; one that shrinks keeps its place. Alignments above 64 are not provided.

In the standard optimized implementation, `global_statistics` returns the documented dummy aggregate, while `heap_statistics` reports the heap’s cached size/span byte counts. The separate statistics-enabled variant adds the listed counter fields; counters conditioned on `ENABLE_STATISTICS` have no values when that option is disabled. The API is tied to the target operating system’s virtual-memory and C runtime facilities.


Platform coverage note: exports for PS5 and Switch 2 targets are unspecified.
