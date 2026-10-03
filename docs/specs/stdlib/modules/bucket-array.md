# Bucket_Array

## Public declaration inventory

Visible public declarations and module-conditional variants are listed below. Procedure bodies and aggregate initializers are omitted; complete public type members are listed separately.

- `Bucket_Array :: struct (type: Type, items_per_bucket: int, always_iterate_by_pointer := false)`
- `Bucket :: struct (type: Type, items_per_bucket: int)`
- `Bucket_Locator :: struct`
- `operator *[] :: (using array: Bucket_Array, index: int) -> *array.type`
- `bucket_array_reset :: (using array: *Bucket_Array)`
- `bucket_array_add :: (using array: *Bucket_Array, item: array.type) -> (Bucket_Locator, pointer: *array.type)`
- `bucket_array_find :: (using array: *Bucket_Array, locator: Bucket_Locator) -> array.type`
- `bucket_array_find_pointer :: (using array: *Bucket_Array, locator: Bucket_Locator) -> *array.type`
- `bucket_array_remove :: (using array: *Bucket_Array, locator: Bucket_Locator)`
- `find_and_occupy_empty_slot :: (using array: *Bucket_Array, $initialize := true) -> *array.type, Bucket_Locator`
- `for_expansion :: (array: *$T/Bucket_Array, body: Code, flags: For_Flags) #expand`

## Public type members

#### `Bucket_Array` (struct)

- `count: s64`
- `allocator: Allocator`
- `My_Bucket :: Bucket(type, items_per_bucket)`
- `all_buckets:    [..] *My_Bucket`
- `unfull_buckets: [..] *My_Bucket`

#### `Bucket` (struct)

- `occupied: [items_per_bucket] bool`
- `data:     [items_per_bucket] type = ---`
- `lowest_maybe_not_occupied: s32`
- `bucket_index: u32`
- `count := 0`

#### `Bucket_Locator` (struct)

- `bucket_index: u32`
- `slot_index:   s32`

## Behavioral contract and constraints

`Bucket_Array(type, items_per_bucket)` grows by allocating fixed-size buckets; existing bucket storage and live element pointers do not move when another bucket is added. `bucket_array_add` occupies a free slot, stores the given value, increments the live count, and returns its physical `Bucket_Locator` plus pointer. `find_and_occupy_empty_slot` performs the same slot reservation without assigning a caller value; with its default `$initialize := true`, it invokes the type initializer or zeroes the slot, and with false the slot remains uninitialized until the caller writes it. Bucket exhaustion creates a bucket, but allocation/exhaustion failure handling is incomplete and may assert. A locator is `(bucket_index, slot_index)`, not a dense iteration index; lookup/removal assert the slot is currently occupied. Removal decrements counts and makes the slot reusable, with no generation check, so stale locators can later designate a different value. Reset frees every bucket and invalidates all pointers/locators.

`for_expansion` is the custom loop expansion over occupied elements. Its emitted `it_index` has type `int` and counts live elements in traversal order: forward iteration yields `0` through `count - 1`; reverse iteration initializes from `count` and decrements before each yielded element, so it yields `count - 1` through `0`. This index is independent of the bucket and slot indices in a locator. The emitted `it` has type `type` by default and type `*type` when the loop uses `For_Flags.POINTER` or `always_iterate_by_pointer` is true; either setting requests pointer iteration. `For_Flags.REVERSE` traverses buckets and slots in reverse order.

The expansion supports the loop-body `remove it` control by supplying a removal-code replacement to the inserted `Code`. That replacement identifies the currently yielded bucket and slot, then applies `bucket_array_remove`; it does not remove from a dense backing array or use `it_index` as a locator. Removal clears the slot's occupied state, decrements both bucket and array live counts, updates the next-free search hint, and makes the slot reusable. If the slot was in a full bucket, that bucket becomes eligible for allocation again. The remaining elements do not move, so continuing iteration preserves their bucket/slot locations; the emitted dense `it_index` counts yielded live items and is not a stable identifier after removal. This behavior applies to value and pointer iteration. A compatible `for_expansion` must provide the `remove` replacement for the language's `remove it` form to work; removal support is an explicit part of each custom expansion's contract, not a generic mutation of its backing storage.

`operator *[] :: (using array: Bucket_Array, index: int) -> *array.type` exposes pointer-based indexing. It computes the bucket and slot for the flattened index, asserts that the slot is occupied, and returns its address. The index must identify an occupied slot; it is not a dense rank among live elements when removals have left holes. This operator enables both ordinary indexing and address-taking through Jai's `*[]` operator rules.

The module leaves bucket-exhaustion error handling incomplete. It does not define a concurrent-access contract.
