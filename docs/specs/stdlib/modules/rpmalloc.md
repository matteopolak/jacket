# rpmalloc


## Public declaration inventory

Only names visible through the module’s exported scope are listed. Declarations from conditional branches retain their condition label; implementation-local declarations are omitted. Aggregate fields and enum members are recorded in the dedicated type-shape section.

- `rpmalloc_allocator_proc :: (mode: Allocator_Mode, requested_size: s64, old_size: s64, old_memory: *void, allocator_data: *void) -> *void`
- `initialize :: () -> s32`
- `initialize :: () -> _int`
- `finalize :: ()`
- `initialize_config :: (config: *rpmalloc_config_t) -> s32`
- `initialize_config :: (config: *rpmalloc_config_t) -> _int`
- `is_initialized :: inline () -> bool`
- `thread_initialize :: inline ()`
- `thread_finalize :: (release_caches: _int)`
- `thread_finalize :: (release_caches: bool)`
- `thread_finalize :: inline (release_caches: bool)`
- `is_thread_initialized :: () -> bool`
- `set_main_thread :: ()`
- `config :: () -> *rpmalloc_config_t`
- `rpmalloc :: inline (size: s64) -> *void`
- `rpfree :: inline (ptr: *void)`
- `rprealloc :: inline (ptr: *void, size: s64) -> *void`
- `rpaligned_realloc :: (ptr: *void, alignment: s64, size: s64, oldsize: s64, flags: Realloc_Flags) -> *void`
- `rpaligned_alloc :: (alignment: s64, size: s64) -> *void`
- `usable_size :: inline (ptr: *void) -> s64`
- `usable_size :: inline (ptr: *void) -> size_t`
- `thread_statistics :: (stats: *rpmalloc_thread_statistics_t)`
- `global_statistics :: (stats: *rpmalloc_global_statistics_t)`
- [ENABLE_STATISTICS] `dump_statistics :: (builder: *$T)`
- [else branch] `dump_statistics :: (builder: *void) #expand`
- `dump_statistics :: (file: *void)`
- `heap_acquire :: inline () -> *Heap`
- `heap_acquire :: inline () -> *rpmalloc_heap_t`
- `heap_release :: inline (heap: *Heap)`
- `heap_release :: inline (heap: *rpmalloc_heap_t)`
- `heap_is_this_yours :: (heap: *Heap, memory: *void) -> bool`
- `is_in_span :: inline (memory: *void, span: *Span) -> bool`
- `heap_alloc :: inline (heap: *Heap, size: s64) -> *void`
- `heap_alloc :: inline (heap: *rpmalloc_heap_t, size: size_t) -> *void`
- `heap_aligned_alloc :: inline (heap: *Heap, alignment: s64, size: s64) -> *void`
- `heap_aligned_alloc :: inline (heap: *rpmalloc_heap_t, alignment: size_t, size: size_t) -> *void`
- `heap_realloc :: inline (heap: *Heap, ptr: *void, size: s64, flags: Realloc_Flags) -> *void`
- `heap_realloc :: inline (heap: *rpmalloc_heap_t, ptr: *void, size: size_t, flags: Realloc_Flags) -> *void`
- `heap_aligned_realloc :: inline (heap: *Heap, ptr: *void, alignment: s64, size: s64, flags: Realloc_Flags) -> *void`
- `heap_aligned_realloc :: inline (heap: *rpmalloc_heap_t, ptr: *void, alignment: size_t, size: size_t, flags: Realloc_Flags) -> *void`
- `heap_free :: inline (heap: *Heap, ptr: *void)`
- `heap_free :: inline (heap: *rpmalloc_heap_t, ptr: *void)`
- `heap_free_all :: inline (heap: *Heap)`
- `heap_free_all :: inline (heap: *rpmalloc_heap_t)`
- `heap_thread_set_current :: inline (heap: *Heap)`
- `heap_thread_set_current :: inline (heap: *rpmalloc_heap_t)`
- `errno : enum s32`
- `rpmalloc_global_statistics_t :: struct`
- `rpmalloc_thread_statistics_t :: struct`
- `rpmalloc_config_t :: struct`
- `rpmalloc_initialize :: () -> _int`
- `rpmalloc_is_initialized :: inline () -> bool`
- `rpmalloc_initialize_config :: (config: *rpmalloc_config_t) -> _int`
- `rpmalloc_finalize :: ()`
- `rpmalloc_thread_initialize :: inline ()`
- `rpmalloc_thread_finalize :: (release_caches: _int)`
- `rpmalloc_is_thread_initialized :: () -> bool`
- `rpmalloc_config :: () -> *rpmalloc_config_t`
- `rpmalloc :: inline (size: size_t) -> *void`
- `rpcalloc :: inline (num: size_t, size: size_t) -> *void`
- `rprealloc :: inline (ptr: *void, size: size_t) -> *void`
- `rpaligned_realloc :: (ptr: *void, alignment: size_t, size: size_t, oldsize: size_t, flags: Realloc_Flags) -> *void`
- `rpaligned_alloc :: (alignment: size_t, size: size_t) -> *void`
- `rpaligned_calloc :: inline (alignment: size_t, num: size_t, size: size_t) -> *void`
- `rpmemalign :: inline (alignment: size_t, size: size_t) -> *void`
- `rpposix_memalign :: inline (memptr: **void, alignment: size_t, size: size_t) -> _int`
- `rpmalloc_usable_size :: inline (ptr: *void) -> size_t`
- `rpmalloc_thread_collect :: inline ()`
- `rpmalloc_thread_statistics :: (stats: *rpmalloc_thread_statistics_t)`
- `rpmalloc_global_statistics :: (stats: *rpmalloc_global_statistics_t)`
- `rpmalloc_dump_statistics :: (file: *void)`
- [RPMALLOC_FIRST_CLASS_HEAPS] `rpmalloc_heap_acquire :: inline () -> *rpmalloc_heap_t`
- [RPMALLOC_FIRST_CLASS_HEAPS] `rpmalloc_heap_release :: inline (heap: *rpmalloc_heap_t)`
- [RPMALLOC_FIRST_CLASS_HEAPS] `rpmalloc_heap_alloc :: inline (heap: *rpmalloc_heap_t, size: size_t) -> *void`
- [RPMALLOC_FIRST_CLASS_HEAPS] `rpmalloc_heap_aligned_alloc :: inline (heap: *rpmalloc_heap_t, alignment: size_t, size: size_t) -> *void`
- [RPMALLOC_FIRST_CLASS_HEAPS] `rpmalloc_heap_calloc :: inline (heap: *rpmalloc_heap_t, num: size_t, size: size_t) -> *void`
- [RPMALLOC_FIRST_CLASS_HEAPS] `rpmalloc_heap_aligned_calloc :: inline (heap: *rpmalloc_heap_t, alignment: size_t, num: size_t, size: size_t) -> *void`
- [RPMALLOC_FIRST_CLASS_HEAPS] `rpmalloc_heap_realloc :: inline (heap: *rpmalloc_heap_t, ptr: *void, size: size_t, flags: Realloc_Flags) -> *void`
- [RPMALLOC_FIRST_CLASS_HEAPS] `rpmalloc_heap_aligned_realloc :: inline (heap: *rpmalloc_heap_t, ptr: *void, alignment: size_t, size: size_t, flags: Realloc_Flags) -> *void`
- [RPMALLOC_FIRST_CLASS_HEAPS] `rpmalloc_heap_free :: inline (heap: *rpmalloc_heap_t, ptr: *void)`
- [RPMALLOC_FIRST_CLASS_HEAPS] `rpmalloc_heap_free_all :: inline (heap: *rpmalloc_heap_t)`
- [RPMALLOC_FIRST_CLASS_HEAPS] `rpmalloc_heap_thread_set_current :: inline (heap: *rpmalloc_heap_t)`
- `ENABLE_OVERRIDE :: false`
- `rpmalloc_linker_reference :: ()`
- `RPMALLOC_CONFIGURABLE :: CONFIGURABLE`
- `Realloc_Flags :: enum_flags u32`
- [RPMALLOC_FIRST_CLASS_HEAPS] `rpmalloc_heap_t :: heap_t`
- `rpmalloc_thread_finalize :: inline (release_caches: bool)`
- `Heap :: heap_t`
- `initialize :: rpmalloc_initialize`
- `initialize_config :: rpmalloc_initialize_config`
- `finalize :: rpmalloc_finalize`
- `is_initialized :: rpmalloc_is_initialized`
- `thread_initialize :: rpmalloc_thread_initialize`
- `thread_finalize :: rpmalloc_thread_finalize`
- `is_thread_initialized :: rpmalloc_is_thread_initialized`
- `set_main_thread :: rpmalloc_set_main_thread`
- `config :: rpmalloc_config`
- `usable_size :: rpmalloc_usable_size`
- `thread_statistics :: rpmalloc_thread_statistics`
- `global_statistics :: rpmalloc_global_statistics`
- `dump_statistics :: rpmalloc_dump_statistics`
- [RPMALLOC_FIRST_CLASS_HEAPS] `heap_acquire :: rpmalloc_heap_acquire`
- [RPMALLOC_FIRST_CLASS_HEAPS] `heap_release :: rpmalloc_heap_release`
- [RPMALLOC_FIRST_CLASS_HEAPS] `heap_alloc :: rpmalloc_heap_alloc`
- [RPMALLOC_FIRST_CLASS_HEAPS] `heap_aligned_alloc :: rpmalloc_heap_aligned_alloc`
- [RPMALLOC_FIRST_CLASS_HEAPS] `heap_realloc :: rpmalloc_heap_realloc`
- [RPMALLOC_FIRST_CLASS_HEAPS] `heap_aligned_realloc :: rpmalloc_heap_aligned_realloc`
- [RPMALLOC_FIRST_CLASS_HEAPS] `heap_free :: rpmalloc_heap_free`
- [RPMALLOC_FIRST_CLASS_HEAPS] `heap_free_all :: rpmalloc_heap_free_all`
- [RPMALLOC_FIRST_CLASS_HEAPS] `heap_thread_set_current :: rpmalloc_heap_thread_set_current`


## Module parameters

These are import-time options, not exported globals.

```jai
#module_parameters (
    HEAP_ARRAY_SIZE         := 47,
    DEFAULT_SPAN_MAP_COUNT  := 64,
    GLOBAL_CACHE_MULTIPLIER := 8,
    ENABLE_THREAD_CACHE := true,
    ENABLE_GLOBAL_CACHE := true,
    ENABLE_STATISTICS    := true,
    ENABLE_ASSERTS       := true,
    ENABLE_VALIDATE_ARGS := true,
    LINUX_FREE_ADVICE : enum { DONTNEED; FREE; } = .DONTNEED,
    CONFIGURABLE                 := false,
    ENABLE_ADAPTIVE_THREAD_CACHE := false,
    LINUX_HUGE_PAGES             := false,
    USE_RAW                      := false,
    DISABLE_UNMAP                := false,
    ENABLE_UNLIMITED_CACHE       := false
)();
```

Each imported rpmalloc module instance owns its allocator state. Imports with the same module and identical parameter argument lists resolve to the same module instance and share its initialization, heaps, and configuration. Different parameter argument lists create separate module instances; for example, two imports with distinct `HEAP_ARRAY_SIZE`, `DEFAULT_SPAN_MAP_COUNT`, or `GLOBAL_CACHE_MULTIPLIER` values have independent allocator initialization and ownership state, and each may be initialized and finalized independently. This applies to both `USE_RAW` branches in the supplied module: the raw branch is a Jai source implementation loaded into the instance, not a process-global external allocator handle. Repeating initialization on an already initialized instance follows that instance's existing initialized behavior and does not replace its configuration. Do not treat imports with different argument lists as aliases merely because their effective defaults happen to match.
## Public aggregate shapes and enum values

Each block gives the exported type declaration with its member fields or enum values. Conditional declarations with distinct shapes are shown as separate variants; platform labels describe availability, not source organization.

### `LINUX_FREE_ADVICE`

```jai
LINUX_FREE_ADVICE : enum { DONTNEED; FREE; }
```
### `Realloc_Flags`

```jai
Realloc_Flags :: enum_flags u32 {
    NO_PRESERVE :: 1;
    GROW_OR_FAIL :: 2;
}
```
### `errno`

```jai
errno : enum s32 {
    SUCCESS ::  0;
    ENOMEM ::  12;
    EINVAL ::  22;
}
```
### `rpmalloc_config_t`

Variant 1 (`USE_RAW := false`; Jai rewrite):

```jai
rpmalloc_config_t :: struct {

	memory_map : #type (size: s64, offset: *s64) -> *void;

	memory_unmap : #type (address: *void, size: s64, offset: s64, release: s64) -> ();

    error_callback : #type (message: string, location: Source_Code_Location) -> ();

	map_fail_callback : #type (size: s64) -> s32;

    page_size: s64;

    span_size: s64;

    span_map_count: s64;

	enable_huge_pages: bool;

	page_name: string;
	huge_page_name: string;
}
```
Variant 2 (`USE_RAW := true`; raw transliteration):

```jai
rpmalloc_config_t :: struct {

	memory_map : #type (size: size_t, offset: *size_t) -> *void;

	memory_unmap : #type (address: *void, size: size_t, offset: size_t, release: size_t) -> ();

    error_callback : #type (message: string) -> ();

	map_fail_callback : #type (size: size_t) -> _int;

    page_size: size_t;

    span_size: size_t;

    span_map_count: size_t;

	enable_huge_pages: _int;

	page_name: string;
	huge_page_name: string;
}
```
### `rpmalloc_global_statistics_t`

Variant 1 (`USE_RAW := false`; Jai rewrite):

```jai
rpmalloc_global_statistics_t :: struct {
	mapped:          s64;
    mapped_peak:     s64;
    cached:          s64;
    huge_alloc:      s64;
    huge_alloc_peak: s64;
    mapped_total:    s64;
    unmapped_total:  s64;
}
```
Variant 2 (`USE_RAW := true`; raw transliteration):

```jai
rpmalloc_global_statistics_t :: struct {

	mapped: size_t;

    mapped_peak: size_t;

    cached: size_t;

    huge_alloc: size_t;

    huge_alloc_peak: size_t;

    mapped_total: size_t;

    unmapped_total: size_t;
}
```
### `rpmalloc_thread_statistics_t`

Variant 1 (`USE_RAW := false`; Jai rewrite):

```jai
rpmalloc_thread_statistics_t :: struct {
    sizecache:        s64;
    spancache:        s64;
    thread_to_global: s64;
    global_to_thread: s64;

	span_use : [64] struct {
	    current:       s64;
	    peak:          s64;
	    to_global:     s64;
	    from_global:   s64;
	    to_cache:      s64;
	    from_cache:    s64;
	    to_reserved:   s64;
	    from_reserved: s64;
	    map_calls:     s64;
	};

	size_use : [128] struct {
	    alloc_current:       s64;
	    alloc_peak:          s64;
	    alloc_total:         s64;
	    free_total:          s64;
	    spans_to_cache:      s64;
	    spans_from_cache:    s64;
	    spans_from_reserved: s64;
	    map_calls:           s64;
	};
}
```
Variant 2 (`USE_RAW := true`; raw transliteration):

```jai
rpmalloc_thread_statistics_t :: struct {

    sizecache: size_t;

    spancache: size_t;

    thread_to_global: size_t;

    global_to_thread: size_t;

	span_use : [64] struct {

	    current: size_t;

	    peak: size_t;

	    to_global: size_t;

	    from_global: size_t;

	    to_cache: size_t;

	    from_cache: size_t;

	    to_reserved: size_t;

	    from_reserved: size_t;

	    map_calls: size_t;
	};

	size_use : [128] struct {

	    alloc_current: size_t;

	    alloc_peak: size_t;

	    alloc_total: size_t;

	    free_total: size_t;

	    spans_to_cache: size_t;

	    spans_from_cache: size_t;

	    spans_from_reserved: size_t;

	    map_calls: size_t;
	};
}
```
## Contextual heap handle layouts

These are the complete layouts of the heap records reachable through the public heap procedures. The rewrite's `Heap` name is module-private; this section describes its required public-signature context without exporting that private name as an additional module declaration. The raw variant's `rpmalloc_heap_t` is available only when first-class heaps are enabled.

### Jai rewrite heap (`USE_RAW := false`)

`Heap` has `owner_thread: u64`, `size_class: [126] Heap_Size_Class`, `span_free_deferred: *void`, `full_span_count: s64`, `span_reserve: *Span`, `span_reserve_master: *Span`, `spans_reserved: u32`, `child_count: s32`, `next_heap: *Heap`, `next_orphan: *Heap`, `id: s32`, `finalize: s32`, `master_heap: *Heap`, `full_span: [126] *Span`, and `large_huge_span: *Span`. With `ENABLE_THREAD_CACHE`, it additionally has `span_cache: Span_Cache` and `span_large_cache: [62] Span_Large_Cache`. With `ENABLE_ADAPTIVE_THREAD_CACHE || ENABLE_STATISTICS`, it has `span_use: [63] Span_Use`; with `ENABLE_STATISTICS`, it also has `size_class_use: [127] Size_Class_Use`, `thread_to_global: s64`, and `global_to_thread: s64`.

`Heap_Size_Class` has `free_list: *void`, `partial_span: *Span`, and `cache: *Span`. `Span_Cache` has `count: s64` and `span: [400] *Span`; `Span_Large_Cache` has `count: s64` and `span: [100] *Span`.

`Span` has `free_list: *void`, `block_count`, `size_class`, `free_list_limit`, `used_count`, `list_size`, `block_size`, `span_count`, `total_spans`, `offset_from_master`, `remaining_spans`, and `align_offset` (each `u32` except `remaining_spans: s32`); `free_list_deferred: *void`; `flags: enum_flags u32 { MASTER :: 1; SUBSPAN :: 2; ALIGNED_BLOCKS :: 4; UNMAPPED_MASTER :: 8; }`; `heap: *Heap`; and `next`, `prev: *Span`.

`Span_Use` has `current, high: s32`; under `ENABLE_STATISTICS` it additionally has `spans_deferred`, `spans_to_global`, `spans_from_global`, `spans_to_cache`, `spans_from_cache`, `spans_to_reserved`, `spans_from_reserved`, and `spans_map_calls` (`s32` each). `Size_Class_Use` (only under `ENABLE_STATISTICS`) has `alloc_current`, `alloc_peak`, `alloc_total`, `free_total`, `spans_current`, `spans_peak`, `spans_to_cache`, `spans_from_cache`, `spans_from_reserved`, `spans_map_calls`, and `unused` (`s32` each).

### Raw heap (`USE_RAW := true`)

The public `rpmalloc_heap_t` aliases the raw `heap_t`. It has `owner_thread: u64`, `size_class: [126] heap_size_class_t`, `span_free_deferred: atomicptr_t`, `full_span_count: size_t`, `span_reserve: *span_t`, `span_reserve_master: *span_t`, `spans_reserved: u32`, `child_count: atomic32_t`, `next_heap`, `next_orphan: *heap_t`, `id: int32_t`, `finalize: _int`, and `master_heap: *heap_t`. Under `ENABLE_THREAD_CACHE`, it additionally has `span_cache: span_cache_t` and `span_large_cache: [62] span_large_cache_t`. Under `RPMALLOC_FIRST_CLASS_HEAPS`, it has `full_span: [126] *span_t` and `large_huge_span: *span_t`. Under `ENABLE_ADAPTIVE_THREAD_CACHE || ENABLE_STATISTICS`, it has `span_use: [63] span_use_t`; under `ENABLE_STATISTICS`, it additionally has `size_class_use: [127] size_class_use_t`, `thread_to_global: atomic64_t`, and `global_to_thread: atomic64_t`.

`heap_size_class_t` has `free_list: *void`, `partial_span: *span_t`, and `cache: *span_t`. `span_cache_t` has `count: size_t` and `span: [400] *span_t`; `span_large_cache_t` has `count: size_t` and `span: [100] *span_t`. `span_t` has the same member order and names as `Span`, with `free_list_deferred: atomicptr_t`, `remaining_spans: atomic32_t`, `heap: *heap_t`, and `next`, `prev: *span_t`; its `flags: u32` uses the bit values listed for the rewrite's `Span.flags`. `span_use_t` and `size_class_use_t` have the same field names as `Span_Use` and `Size_Class_Use`, with `atomic32_t` for counters except `high`, `alloc_peak`, and `spans_peak`, which are `int32_t`.

## Heap and span state access

Heap and span records are exposed so callers can inspect the allocator state reached through the heap APIs. Their fields are allocator-owned control and accounting state. The module defines no supported operation for callers to write these fields directly, and public field visibility alone does not make edits valid. Mutations are made through allocator operations such as allocation, free, resize, thread finalization, setting the current heap, and heap release. The module does not promise that a direct field edit is applied to native metadata or survives the next allocator operation; it may corrupt allocator state.

Heap handles returned by `heap_acquire` remain usable only while the acquired heap is live. `heap_release` ends that lifetime; clients must not read or pass the pointer afterward, and address reuse does not preserve the former handle's identity. Span pointers are transient allocator-managed references: allocation, free, cache movement, reserve use, heap release, and unmapping can change their membership or lifetime. There is no public span-acquisition or span-lifetime lease API. Keep a live heap and avoid concurrent mutation while inspecting its non-atomic fields. No coherent multi-field snapshot or synchronization guarantee is provided for direct reads. Raw-profile atomic members retain their native atomic representation; reading them requires appropriate atomic access, while the rewrite's plain scalar fields do not create a snapshot API.

In the rewrite, `heap_thread_set_current` selects the supplied heap for the calling thread and releases that thread's previous heap when it differs. A caller must treat the previous heap handle as released after replacement. This procedure is an allocator operation; changing `owner_thread` or related fields directly is not an equivalent supported operation.

Only entries within a cache's `count` are active cache entries. Array elements outside that prefix have no specified value and must not be followed or treated as null. `span_reserve` and `span_reserve_master` identify allocator-managed reserve state when reserve spans are present; `spans_reserved` is the associated count. With no reserved spans, the pointer fields have no documented nulling guarantee. A reserved span may be mapped but unused and is not a general live allocation-span handle. Do not recursively inspect unoccupied cache entries, inactive reserve storage, or freed/unmapped spans.

For the `partial_span` and `full_span` heads, the module documents doubly linked lists of partial/full spans, with a nonempty head's `prev` link serving as the tail link. `next` links traverse current membership. Treat this as a convention for inspecting a stable, active list, not a general graph traversal or caller mutation API: no null-value guarantee is made for inactive/stale links, and list heads during concurrent mutation are not stable snapshots. `free_list` and deferred-free pointers are allocator-managed metadata and are not caller-owned lists to edit.

In the rewrite, `heap_is_this_yours` reports whether an address is within a span reachable from the selected heap's current partial/full size-class lists or its large/huge list. It is an allocator ownership-range query, not validation that the address is an allocated block start or is still live; addresses in inactive caches/reserve state are not reported as owned by this query. `is_in_span` checks the span's mapped address range and does not validate an individual allocation. These results are meaningful only while the heap/span metadata is live and not being concurrently changed.

Public contract tests can check the reflected public shapes and conditional fields, then exercise real allocation/free/resize/current-heap/release calls and query ownership at quiescent points. With statistics enabled, tests can check documented counter changes after those operations. Tests should not prescribe direct metadata writes, inactive cache contents, reserve nullness, stale pointer behavior after release, or private graph organization beyond the documented active list links.

## Generic allocator procedure

`rpmalloc_allocator_proc` implements the `Preload.Allocator_Mode` callback. Its `allocator_data` is either null (use rpmalloc's default thread heap for allocation, resize, free, and ownership checks) or a heap handle returned by `CREATE_HEAP`; callers own the handle lifecycle and pass it back for its operations. `old_size` is not consulted. The return value is the allocation pointer for `ALLOCATE`/`RESIZE`, null for operations without a result, an acquired heap for `CREATE_HEAP`, a pointer-encoded boolean for `IS_THIS_YOURS`, and pointer-encoded capability bits for `CAPS`.

| Mode | Observable behavior |
| --- | --- |
| `ALLOCATE` | Allocate `requested_size` bytes from the selected heap and return the pointer. |
| `RESIZE` | Resize `old_memory` to `requested_size` on the selected heap and return the resulting pointer. |
| `FREE` | Free `old_memory` on the selected heap; return null. |
| `STARTUP` / `SHUTDOWN` | Initialize / finalize the allocator; return null. The procedure ignores the initialization result. |
| `THREAD_START` / `THREAD_STOP` | Initialize the calling thread / finalize it with cache release enabled; return null. |
| `CREATE_HEAP` | Acquire and return a heap handle. |
| `DESTROY_HEAP` | Release the heap in `allocator_data`; null is an assertion failure because the default heap cannot be destroyed. |
| `IS_THIS_YOURS` | Check whether `old_memory` falls within a span owned by the selected heap; return 1 or 0 encoded as `*void`. |
| `CAPS` | If `old_memory` is non-null, treat it as writable `*string` storage and assign the string value `rpmalloc 1.4.4`. Return the flags `HINT_I_AM_A_GENERAL_HEAP_ALLOCATOR`, `MULTIPLE_THREADS`, `FREE`, `ACTUALLY_RESIZE`, and `CREATE_HEAP` encoded as `*void`. |

The `CAPS` result does not include `IS_THIS_YOURS`, although the procedure handles that mode. Treat the returned flags as the declared capabilities; the procedure's additional mode handling does not change the flags. Invalid mode values are reported through the active assertion mechanism and return null. Allocation and resize failure behavior follows the selected allocator entry point; this callback does not translate a null result into another error channel.

## Statistics output

In the rewrite, with `ENABLE_STATISTICS`, `dump_statistics(builder)` accepts only `*Basic.String_Builder` (the generic parameter is compile-time checked) and appends a human-readable snapshot of per-heap allocation/span counters and global mapping/cache counters to the builder's existing contents. It does not clear the builder. Output formatting is diagnostic text rather than a stable machine-readable serialization. With statistics disabled, the exported builder form expands to an empty operation and has no output side effect.

In the raw variant, the exported `dump_statistics(file: *void)` uses the argument as a C `FILE *` for text output. The module gates that output on both `ENABLE_STATISTICS` and `PRETTY_PRINT`; `PRETTY_PRINT` is not a declared public module parameter, so the public module interface does not guarantee a supported way to select that condition. The raw file form and rewrite builder form are distinct conditional APIs, not overloads with a shared output contract.

## Contracts and constraints

This page binds the rpmalloc allocator and heap API. `USE_RAW := false` selects the Jai rewrite; `USE_RAW := true` selects the raw transliteration and changes public config/statistics field types. The config and statistics blocks above are conditional variants, not interchangeable layouts. In particular, config callbacks in the rewrite use `s64` sizes and a `Source_Code_Location` error callback, while the raw variant uses `size_t` and a one-argument error callback; callers must select the variant matching the imported module parameter and must not pass one variant's record to the other.

`CONFIGURABLE` defaults false; custom `page_size`, `span_size`, and `span_map_count` settings take effect only for configurable initialization. `page_size` must be a power of two; `span_size` is either zero for the implementation default or a power of two in the supported 4096–262144 range. Mapping and unmapping callbacks are used as a pair: if either is absent, the implementation selects both built-in callbacks. When supplied, both callbacks may be invoked concurrently and must be thread-safe. Mapped addresses must meet span alignment, and the returned offset must be representable by the API. `map_fail_callback` returning zero declines retry and causes allocation failure to return null; nonzero requests a retry. A failed allocation is not translated into a Jai allocator error. Allocations belong to rpmalloc and must be freed by its matching API; thread/global initialization and teardown must follow the allocator lifecycle.

`RPMALLOC_FIRST_CLASS_HEAPS` in the raw variant aliases the compile-time name `FIRST_CLASS_HEAPS` and controls whether the raw heap-specific C-style names and `rpmalloc_heap_t` are available. The module's declared parameter list does not declare `FIRST_CLASS_HEAPS`, its type, or a default. The module therefore establishes no public selection value or default for this raw-only conditional API. A private build setting with a false default is an implementation packaging choice, not an additional public module parameter or a source-defined compatibility rule. The pinned native C header's default of zero likewise does not guarantee the missing Jai import-time value. The rewrite's separate heap procedures and contextual `Heap` contract are documented independently. Custom cache/statistics/assertion and Linux page advice options alter behavior but do not make the two config layouts ABI-compatible. The raw and rewrite implementations are distinct public variants; the module does not claim the raw record callback spellings are native C ABI-compatible with the Jai rewrite's callback types. The library/build files are platform-specific.

Both `initialize_config` variants copy the supplied record into module-owned configuration storage; the caller may release the record itself after the call. `config()` returns a mutable pointer to that storage, not the original input. Callback procedure values are retained there and may be called from multiple threads; mapping and unmapping callbacks must be thread-safe. The config's Jai `string` fields are shallow-copied, so their referenced bytes must remain valid while the allocator may use them. Callback message strings are borrowed for the duration of the callback. The rewrite error callback receives the `Source_Code_Location` passed to the failing assertion; ordinarily this identifies the assertion site, not necessarily the allocation call that led there. The raw callback receives only the message, so it has no source-location argument. `map_fail_callback` applies only when the default OS mapping callback is selected; it receives the attempted mapping byte count (including alignment padding when the request uses it), and a nonzero result asks the allocator to retry. If it is absent or returns zero, the allocation returns null.

The map/unmap `offset` is a paired alignment-displacement value: the value written by map is passed back to unmap, and it must not exceed 65,535. The public callback comments describe it as an offset in bytes, while the built-in aligned mapping path stores the displacement in 8-byte units and its matching unmap path converts it back to bytes. Custom callback pairs should preserve and interpret the value consistently, and the module does not guarantee a portable custom-callback unit convention beyond the paired-value and range requirements. Rewrite offsets are `s64`; raw offsets are `size_t`, but neither type enlarges the documented 65,535 maximum.

After initialization, `config()` returns the address of the module-owned mutable configuration record. If either mapping callback is absent, initialization selects the default OS map/unmap pair; the error and map-failure callbacks remain unset by default, and page-name strings remain empty unless supplied. The error callback is consulted only when assertions are enabled. Effective `page_size`, `span_size`, `span_map_count`, and huge-page status are normalized during initialization and reflected in this record; their exact numeric values can depend on the target and page configuration. Initialization copies the input record, so the input record itself need not remain alive, but callback code and string data referenced by the copied record must remain valid while used.

Writes through the pointer returned by `config()` update the live stored record. Later mapping, unmapping, assertion, mapping-failure, and page-name uses read the corresponding stored callback or string fields, so changing those fields can affect later operations. Keep map/unmap callbacks a compatible pair for the lifetime of every mapping they create, keep any referenced callback/string storage alive, and do not race configuration writes against allocator use; no synchronization or migration of existing mappings is promised. The sizing inputs are consumed and normalized at initialization: changing `page_size`, `span_size`, `span_map_count`, or `enable_huge_pages` afterward does not reconfigure the allocator's already-derived sizing state. The record's mutability alone does not guarantee reinitialization semantics.

### Configuration-option semantics

`HEAP_ARRAY_SIZE` is the positive compile-time bucket count for the private heap lookup table. The module uses modulo by this count and does not require a power of two; changing it changes internal storage and lookup distribution, not the public heap API or a promised handle identity. `GLOBAL_CACHE_MULTIPLIER` scales the compile-time global span-cache capacity. `DEFAULT_SPAN_MAP_COUNT` is the fallback batch count when the configuration's `span_map_count` is zero. Initialization may raise the effective map batch to cover a page or make it page-size aligned; the resulting effective value is written back to the configuration. The per-heap reserve limit is the lesser of that effective batch count and `DEFAULT_SPAN_MAP_COUNT`. Excess spans can be held in the separate global reserve; the module guarantees no `DEFAULT_SPAN_MAP_COUNT` as a global-reserve cap. The module does not validate a separate numeric range for these compile-time size parameters; positive values are the supported useful domain.

`ENABLE_ADAPTIVE_THREAD_CACHE` requires `ENABLE_THREAD_CACHE`; the module rejects the combination at compile time when adaptive caching is enabled without thread caching. `Heap.span_use` is present when adaptive caching or statistics is enabled. Its `current` and `high` fields describe in-use spans (excluding cached spans); statistics-enabled builds add the transition counters. When both adaptive caching and statistics are disabled, this field is absent from the public conditional layout.

The compile-time cache options have these required combinations: `ENABLE_GLOBAL_CACHE=true` requires `ENABLE_THREAD_CACHE=true`; `ENABLE_GLOBAL_CACHE=false` requires `ENABLE_UNLIMITED_CACHE=false`; `DISABLE_UNMAP=true` requires both global and unlimited caching; and adaptive thread caching requires thread caching. Violating one of these constraints is a compile-time assertion failure. `ENABLE_ASSERTS` and `ENABLE_VALIDATE_ARGS` independently control runtime assertion and public-argument validation code; disabling them changes behavior for invalid calls, which are outside the supported input domain. Valid allocation operations remain the intended use under either setting.

On Linux, `LINUX_HUGE_PAGES` enables the optional `/proc/meminfo` inspection code, but default huge-page sizing is attempted only if the runtime configuration also requests `enable_huge_pages`. If the compile-time option is off, a request with no explicit `page_size` does not activate that inspection. If the option is on but the memory-info file or a usable `Hugepagesize` value is unavailable, initialization silently keeps ordinary page sizing and reports `enable_huge_pages=false` in the stored configuration. An explicit nonzero `page_size` bypasses automatic page-size discovery; in that path the runtime request is copied into the stored huge-page state without the `/proc/meminfo` check. Discovering a size does not promise that returned mappings are backed by physical huge pages: the Linux mapping path does not explicitly request a huge-page mapping flag. Later mapping failure follows the mapping callback/null-failure policy and does not retry with ordinary page sizing. Other target branches have their own native large-page behavior; this Linux option does not define it.

`LINUX_FREE_ADVICE` selects which advice code is passed to `madvise` on the POSIX retained-span path: `DONTNEED` requests the discard/reclaim behavior, while `FREE` requests lazy reclamation. Although named for Linux, the module uses this option in that non-Windows/non-PS5 path. A failed advice call asserts; there is no fallback to the other advice value. When unmapping is disabled, this advice path is not used. The option selects a compile-time constant and does not guarantee that every OS/kernel supports `MADV_FREE`.


Platform coverage note: PS5 branches exist, but their declarations are not specified here; exports for those targets are unspecified.
