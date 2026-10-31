# Hash_Map

An opt-in hash map that keeps one byte of metadata per slot and checks sixteen slots per step with one vector compare (the SwissTable design). Each key is stored once. Import it namespaced:

```jai
Map :: #import "Extensions/Hash_Map";

ages: Map.Map(string, int);
defer Map.deinit(*ages);

previous, replaced := Map.insert(*ages, "ada", 36);  // 0, false
value := Map.find(*ages, "ada");                     // *int, null when missing
age := Map.get(*ages, "ada", default = 0);
slot, added := Map.entry(*ages, "grace");            // *int to a new zeroed value
old, removed := Map.delete(*ages, "ada");

for value, name: ages print("% is %\n", name, value);   // `it_index` is the key
for ages if it < 18 remove it;                          // removal inside the loop is supported
```

`Map.Map(Key, Value, hash, equal)` takes an optional hash `(Key) -> u32` and equality `(Key, Key) -> bool`. A map is zero-initialized and allocates on first insert. Iteration order is not stable, and any insert that rebuilds the map invalidates pointers from `find` and `entry`.

## How it works

- One allocation holds `capacity` control bytes then `capacity` entries. A control byte is `EMPTY` (`0xFF`), `DELETED` (`0x80`) or the top seven bits of the spread hash.
- Lookup starts at the group chosen by the low hash bits and compares all 16 control bytes against the tag in one SIMD compare; the key compare runs only on those candidates. A group with an `EMPTY` slot ends the search; otherwise probing moves by triangular offsets.
- Load limit is 7/8. Removing marks a slot `EMPTY` if its group already has an empty slot, else `DELETED`. When `growth_left` hits zero the map is rebuilt at the same size (dropping tombstones) if at most half the limit is live, otherwise doubled.
- `for` walks groups by occupied-slot bits, skipping empty groups. `remove it` never rebuilds, so the loop stays valid; adding inside the loop is not.
- Group compares exist as SIMD and scalar versions (`SCALAR_GROUPS = true`, or `OS == .WASM`). Both give identical layouts; `tests/agreement.jai` runs them against `Hash_Table`.

```jai
Plain :: #import "Extensions/Hash_Map"(SCALAR_GROUPS = true);
```

## How to change it

- Probe, insert and remove logic is in `module.jai` below `#scope_file` (`find_slot`, `locate`, `claim`, `remove_slot`, `rebuild`). A map with capacity must always keep one `EMPTY` slot or probing never ends; `max_load` guarantees it.
- Changing the group size means changing `GROUP_SIZE`, the vector type and the `u32` masks together.
- A new group scan needs scalar and vector versions and a check in `tests/agreement.jai`.
- Tests: `jacket test --path stdlib/Extensions/Hash_Map` (add `--interp` or the wasm runner for other modes).

## Measured against Hash_Table and the community map

Benchmarks in `benchmarks/compare/hash-map` (`tools/bench/library_compare.py hash-map --per-item`, see [library comparisons](../tools/library-comparisons.md)); keys are random `s64` or strings, at 1k to 1M entries. Ratio is Hash_Table's time over Hash_Map's (above 1 means Hash_Map is faster), Apple M5.

| Workload | 1k | 100k | 1m |
| --- | --- | --- | --- |
| insert | 0.91x | 1.94x | 2.16x |
| hit | 1.35x | 1.12x | 1.05x |
| miss | 1.14x | 1.25x | 3.95x |
| churn | 0.97x | 1.53x | 1.60x |
| iterate | 1.94x | 6.18x | 2.61x |
| string insert | 1.19x | 1.97x | 1.63x |
| string hit | 1.20x | 1.14x | 1.14x |

It wins most once the map outgrows the cache: a miss ends after one group, removed slots are reused, and iteration skips empty groups. Small maps are close to even because they rebuild several times on the way up.
