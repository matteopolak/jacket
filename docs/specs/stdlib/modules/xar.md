# Xar


## Public declaration inventory

Only names visible through the module’s exported scope are listed. Declarations from conditional branches retain their condition label; implementation-local declarations are omitted. Aggregate fields and enum members are recorded in the dedicated type-shape section.

- `Xar :: struct (Element_Type: Type, first_chunk_size_shift: u8, max_chunks: int)`
- `xar_reset :: (using array: *Xar)`
- `xar_add :: (using array: *Xar, el: array.Element_Type) -> *array.Element_Type`
- `xar_unordered_remove :: (using array: *Xar, index: int)`
- `for_expansion :: (using array: *$T/Xar, body: Code, flags: For_Flags) #expand`
- `operator *[] :: inline (using array: Xar, index: int) -> *array.Element_Type`
- `xar_iterator :: inline (using array: *Xar, index: int) -> array.Iterator`
- `xar_next :: (using array: *Xar, iterator: *array.Iterator) -> *array.Element_Type`
- `xar_prev :: (using array: *Xar, iterator: *array.Iterator) -> *array.Element_Type`
- `xar_get :: inline (using array: *Xar, iterator: array.Iterator) -> *array.Element_Type`
- `quick_sort :: quick_sort_iter`
- `quick_sort_simple :: quick_sort_iter`
- `quick_sort_iter :: (array: $T/Xar, f: (array.Element_Type, array.Element_Type) -> $R)`


## Public aggregate shapes and enum values

Each block gives the exported type declaration with its member fields or enum values. Conditional declarations with distinct shapes are shown as separate variants; platform labels describe availability, not source organization.

### `Xar`

```jai
Xar :: struct (Element_Type: Type, first_chunk_size_shift: u8, max_chunks: int) {
    count:     s64;
    chunks:    [max_chunks] *Element_Type;
    allocator: Allocator;

    Iterator :: struct {
        it_index: int;
        index_in_chunk: int;
        chunk_cap: int;
        chunk_pointer: **Element_Type;
    }
}
```
## Contracts and constraints

`Xar(T, first_chunk_size_shift, max_chunks)` has a compile-time fixed capacity of `2^(first_chunk_size_shift + max_chunks - 1)` elements for the intended positive chunk count: chunk capacities are `a, a, 2a, 4a, ...`, where `a = 1 << first_chunk_size_shift`. Element addresses remain stable when later chunks are allocated. `xar_add` appends and returns the address of the new element; it asserts when the index exceeds the configured chunks. The first allocation captures the current allocator for the `Xar`; `xar_reset` frees every allocated chunk through that allocator, clears chunk pointers, and sets `count` to zero. It does not change the stored allocator.

`xar_unordered_remove` requires `index < count`, copies the final live element into the removed slot, and decrements `count`; it does not preserve order or release chunk storage. Pointers to the overwritten slot now refer to the moved value, and the former last slot is outside the live range. `operator *[]` maps an index to its chunk without checking it against `count`; caller must pass a live index. `xar_iterator` likewise expects a valid index. `xar_next` returns null at the end; `xar_prev` returns null at index zero. The `for_expansion` form rejects reverse iteration at compile time. `xar_reset` invalidates every previously returned element pointer because it frees the chunks. The module marks allocation alignment as unfinished, so over-aligned element types are not guaranteed. Behavior for invalid type parameters such as zero chunks or overflowing shifts is not specified.
