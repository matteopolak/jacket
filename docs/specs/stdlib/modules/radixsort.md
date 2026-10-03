# RadixSort


## Public declaration inventory

Only names visible through the module’s exported scope are listed. Declarations from conditional branches retain their condition label; implementation-local declarations are omitted. Aggregate fields and enum members are recorded in the dedicated type-shape section.

- `RadixSort :: struct`
- `reset :: (using radix: *RadixSort)`
- `resize :: (using radix: *RadixSort, new_size: u32)`
- `free_resources :: (using radix: *RadixSort)`
- `operator[] :: (using radix: RadixSort, i: int) -> u32`
- `rank :: (using radix: *RadixSort, i: int) -> u32`
- `sort :: (using radix: *RadixSort, input: *u32, count: u32)`
- `sort :: (using radix: *RadixSort, input: *u64, count: u32)`
- `sort :: (using radix: *RadixSort, input: *float, count: u32)`
- `sort :: (using radix: *RadixSort, input: []u32)`
- `sort :: (using radix: *RadixSort, input: []u64)`
- `sort :: (using radix: *RadixSort, input: []float)`
- `reorder :: (input: *[..]$T, radix: RadixSort)`


- `radix_tests :: ()`

## Public aggregate shapes and enum values

Each block gives the exported type declaration with its member fields or enum values. Conditional declarations with distinct shapes are shown as separate variants; platform labels describe availability, not source organization.

### `RadixSort`

```jai
RadixSort :: struct {

    ranks:  [] u32;
    ranks2: [] u32;

    valid_ranks : bool;

    allocator : Allocator;
}
```
## Behavioral contract

`sort` records indices in sorted order in `ranks`; it does not reorder the input. Overloads accept `u32`, `u64`, and `float` pointers with counts or corresponding array views. Signed integer overloads are not provided. `rank` and `operator[]` read an index from the stored rank array; callers must use an index within its current count and only read ranks after a successful non-empty sort. A null pointer or zero count returns without changing the ranks, including leaving any previously valid ranks intact. Sorting float values restores the input values before return; NaNs are not assigned a documented ordering.

`reset` marks ranks invalid without releasing storage. `resize` changes both rank-array counts and invalidates ranks when the requested size differs; shrinking retains capacity, while growth uses the instance allocator. `free_resources` releases both buffers. `reorder` resizes a new array, copies elements by rank, frees the original resizable array and replaces it; its input must be a valid resizable array, and the array allocator/lifetime rules apply. Dispose of the sorter with `free_resources` when finished. Depends on Basic allocation/array helpers.
