# Sort


## Public declaration inventory

Only names visible through the module’s exported scope are listed. Declarations from conditional branches retain their condition label; implementation-local declarations are omitted. Aggregate fields and enum members are recorded in the dedicated type-shape section.

- `quick_sort :: (array: [] $T, f: (T, T) -> $R) -> [] T`
- `quick_sort :: (array: [] $T, f: (T) -> $R) -> [] T`
- `compare_floats :: (a: float, b: float) -> s64`
- `compare_strings :: (a: string, b: string) -> s64`


## Behavioral contract

`quick_sort` sorts an array view in place and returns that same view. The comparator may compare two values directly or return a key for one value; its result must provide a consistent ordering. Equal items have no stable-order guarantee. The float and string comparators provide the module's built-in ordering choices. Empty and singleton arrays remain valid inputs and need no caller allocation. The module does not define special handling for NaNs or malformed comparator orderings.
