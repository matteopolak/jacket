# Hash_Map (`Extensions/Hash_Map`)

## What it is

An opt-in hash map that keeps one byte of metadata per slot and checks sixteen slots per step with one vector compare (the SwissTable design). It stores each key once. Import it namespaced:

```jai
Map :: #import "Extensions/Hash_Map";

ages: Map.Map(string, int);
defer Map.deinit(*ages);

previous, replaced := Map.insert(*ages, "ada", 36);  // 0, false
previous, replaced = Map.insert(*ages, "ada", 37);   // 36, true
value := Map.find(*ages, "ada");                     // *int, null when missing
age := Map.get(*ages, "ada", default = 0);           // 37
slot, added := Map.entry(*ages, "grace");            // *int to a new zeroed value, true
old, removed := Map.delete(*ages, "ada");            // 37, true

for value, name: ages print("% is %\n", name, value);   // `it_index` is the key
for ages if it < 18 remove it;                          // removal inside the loop is supported
```

| Procedure | Result |
| --- | --- |
| `insert(*map, key, value)` | stores the value; returns the previous value (zero if none) and whether one was replaced |
| `find(*map, key)` | `*Value`, or `null` if the key is missing |
| `get(*map, key, default)` | a copy of the value, or `default` (the zero value unless given) |
| `entry(*map, key)` | `*Value` for the key, adding a zero-initialized one if missing; also whether it was added |
| `delete(*map, key)` | the removed value (zero if none) and whether the key was present |
| `contains(*map, key)` | `bool` |
| `reserve(*map, count)` | room for `count` entries in total without a rebuild |
| `clear(*map)` | removes every entry and keeps the memory |
| `deinit(*map)` | removes every entry and frees the memory; the map is usable again afterwards |

`for` iterates forward, `<` in reverse, `*` by pointer.

`Map.Map(Key, Value, hash, equal)` takes an optional hash procedure `(Key) -> u32` and an optional equality procedure `(Key, Key) -> bool`. A map is zero-initialized and allocates on first insert.

Only `count` and `capacity` are meant to be read. The other fields (`growth_left`, `allocator`, `control`, `entries`) are internal.

A key is stored once, and iteration order is not stable: growing or rebuilding the map reorders it. Pointers from `find` and `entry` are invalidated by any insert that rebuilds the map.

## How it works

One allocation holds `capacity` control bytes followed by `capacity` entries (`key`, `value`). `capacity` is zero, or a power of two of at least 16. A control byte is `EMPTY` (`0xFF`), `DELETED` (`0x80`), or the top seven bits of the key's spread hash (`0..127`), so the high bit says "not holding an entry".

The user's 32-bit hash is multiplied once more and folded (`split_hash`) so a weak hash, such as `key % 4`, still gives a spread starting group (low bits) and a useful tag (top 7 bits).

Lookup starts at the group chosen by the low bits and, for each group, compares all 16 control bytes with the tag in one SIMD compare. Each set bit is a candidate; the key compare runs only on those, which is rare for a miss. If the group has an `EMPTY` slot the key is absent; otherwise probing moves to the next group by triangular offsets (1, 3, 6, ...), which visits every group of a power-of-two map.

Load limit is 7/8 of the slots. `growth_left` counts empty slots that may still be filled. Inserting into a `DELETED` slot does not spend it, which is how delete-heavy workloads stay put. Removing marks a slot `EMPTY` when its group already has an empty slot (no probe ever went past that group), and `DELETED` otherwise. When `growth_left` reaches zero the map is rebuilt: at the same size if at most half the load limit is live (this drops the tombstones), otherwise at double. Rebuilding allocates a new block and reinserts, so it is not in place and briefly needs both blocks. The tests assert that insert/remove churn does not grow the capacity.

`for` iterates by group, taking each group's occupied-slot bits, and skips empty groups whole. `remove it` clears the current slot the same way `delete` does; it never rebuilds, so the loop stays valid. Adding to the map inside the loop does not.

### Scalar and vector paths

The group compares (`match_tag`, `match_empty`, `match_free`) exist twice. The default uses `Extensions/SIMD` (16 lanes of `u8`: SSE2/NEON in optimized native builds, ordinary loops in the interpreter). `OS == .WASM` or the module parameter `SCALAR_GROUPS = true` selects explicit loops instead. Both give identical answers and identical layouts; `tests/agreement.jai` runs them side by side against `Hash_Table` and compares the control bytes.

```jai
Plain :: #import "Extensions/Hash_Map"(SCALAR_GROUPS = true);
```

## How to change it

- The probe, insert and remove logic is in `module.jai` below `#scope_file`: `find_slot` (lookup), `locate` (lookup that also remembers the first reusable slot), `claim`, `remove_slot`, `rebuild`. Keep the invariant that a map with capacity always has at least one `EMPTY` slot, or probing never ends; `max_load` is what guarantees it.
- Changing the group size means changing `GROUP_SIZE`, the SIMD vector type and the `u32` bit masks together.
- A new group scan needs a scalar and a vector version with the same results, and a check in `tests/agreement.jai`.
- Not done: storing entries and control in separate allocations, in-place rehash, shrinking on remove, multiple values per key.
- Tests: `python3 tools/stdlib/stdlib_runtime.py --jaic <jaic> --filter Hash_Map --modes interp,native,wasm-interp`. They cover basics, growth over many rebuilds, tombstone churn, string keys, custom hash and equal, iteration exactly once, removal during iteration, and the scalar/vector agreement.

## Configuration

- `SCALAR_GROUPS` (module parameter, default `false`): use plain loops for the group compares.
- `GROUP_SIZE :: 16` and the 7/8 load limit are constants in `module.jai`.
- `Map` parameters: `Key`, `Value`, optional `hash` `(Key) -> u32`, optional `equal` `(Key, Key) -> bool`. Defaults are `Hash.get_hash` and `==`.
- The allocator is `map.allocator`, defaulting to `context.allocator` at first use.

## Dependencies

`Basic`, `Hash` (default hash) and `Extensions/SIMD`.

## Measured against Hash_Table

`benchmarks/hash-map.jai`, built with `-O2` on an Apple M-series Mac (arm64, NEON), median of seven runs of each workload, `s64` keys unless noted. Speedup is `Hash_Table` time over `Hash_Map` time, so above 1 means `Hash_Map` is faster. Numbers move by 10 to 20 percent from run to run on a busy machine.

| Workload (ns/op, Hash_Table / Hash_Map) | 1k entries | 100k entries | 1M entries |
| --- | --- | --- | --- |
| insert (including growth) | 20.0 / 24.2 (0.83x) | 40.1 / 21.0 (1.9x) | 56.2 / 31.9 (1.8x) |
| lookup hit | 2.7 / 3.2 (0.83x) | 7.3 / 5.8 (1.3x) | 14.0 / 12.7 (1.1x) |
| lookup miss | 3.4 / 3.1 (1.1x) | 12.5 / 7.0 (1.8x) | 29.2 / 4.6 (6.4x) |
| remove + insert (steady size) | 37.5 / 16.8 (2.2x) | 58.6 / 23.4 (2.5x) | 79.9 / 50.4 (1.6x) |
| iterate | 1.5 / 1.4 (1.06x) | 6.4 / 1.3 (4.9x) | 5.7 / 2.0 (2.8x) |
| string insert | 44.2 / 44.6 (0.99x) | 73.4 / 40.6 (1.8x) | 105.3 / 84.5 (1.25x) |
| string lookup hit | 12.0 / 11.5 (1.04x) | 26.3 / 22.0 (1.2x) | 55.1 / 73.0 (0.75x) |

Reading it: `Hash_Map` wins most where the map outgrows the cache (misses end after one group instead of walking a cluster; deleted slots are reused; iteration skips empty groups). At 1k entries insert is slower because small maps rebuild many times (16 slots up) and a rebuild rehashes every key. Hits on a large string map lose a little: a lookup touches the control byte, the entry and then the string, where `Hash_Table` reads the entry and the string.

A word-frequency count over 6 million tokens (200k distinct strings, `entry` then an increment) took about 10 percent less time with `Hash_Map` (roughly 430 to 540 ms against 480 to 650 ms; the machine was loaded). `benchmarks/runtime/stdlib-table.jai` times `Hash_Table` on its own.
