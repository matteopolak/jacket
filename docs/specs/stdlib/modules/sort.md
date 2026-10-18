# Sort


## Public declaration inventory

Only names visible through the module’s exported scope are listed. Declarations from conditional branches retain their condition label; implementation-local declarations are omitted. Aggregate fields and enum members are recorded in the dedicated type-shape section.

- `intro_sort :: (array: [] $T, f: (T, T) -> $R)`
- `intro_sort :: (array: [] $T, key: (T) -> $R)`
- `heap_sort :: (array: [] $T, f: (T, T) -> $R) -> [] T`
- `quick_sort :: (array: [] $T, f: (T, T) -> $R) -> [] T` (deprecated, calls `intro_sort`)
- `quick_sort :: (array: [] $T, f: (T) -> $R) -> [] T` (deprecated, calls `intro_sort`)
- `compare_floats :: (a: float, b: float) -> s64`
- `compare_strings :: (a: string, b: string) -> s64`


## Behavioral contract

`intro_sort` sorts an array view in place: quicksort with a median-of-three pivot that falls back to `heap_sort` when partitioning nests too deep, and finishes short ranges with insertion sort. The two-argument comparator returns negative, zero or positive; equal items have no stable-order guarantee. The key overload calls the key procedure once per element, sorts a temporary `(key, position, value)` array (allocated with the context allocator) and keeps equal keys in their original order. `heap_sort` is the heap algorithm alone and returns the same view. `quick_sort` was the name of both and is deprecated; it forwards to `intro_sort` and will be removed after one release. The float and string comparators provide the module's built-in ordering choices. Empty and singleton arrays remain valid inputs and need no caller allocation. The module does not define special handling for NaNs or malformed comparator orderings.
