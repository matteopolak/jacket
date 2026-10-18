# IntroSort

## Public declaration inventory

Visible public declarations and module-conditional variants are listed below. Procedure bodies and aggregate initializers are omitted; complete public type members are listed separately.

- `intro_sort :: (array: [] $T, f: (T, T) -> $R)`
- `insertion_sort :: (array: [] $T, f: (T, T) -> $R)`

## Public type members

No public aggregate fields or enum members were identified.

## Behavioral contract and constraints

`intro_sort` sorts the supplied array view in place using the comparison callback (the same procedure as `Sort.intro_sort`, which also has a key overload); negative/zero/positive results define less/equal/greater ordering. `insertion_sort` exposes the same mutation and comparator contract. Neither returns a separate sorted array, and stable ordering of equal elements is not promised. The callback must be consistent and the array view must describe valid storage. No error result is defined for invalid storage or a contradictory comparator.
