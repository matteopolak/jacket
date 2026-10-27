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

The default hash of a `string` key is the module's own `hash_text` (eight bytes per step, length included); every other key type uses `Hash.get_hash`. The user's 32-bit hash is multiplied once more and folded (`split_hash`) so a weak hash, such as `key % 4`, still gives a spread starting group (low bits) and a useful tag (top 7 bits).

Lookup starts at the group chosen by the low bits and, for each group, compares all 16 control bytes with the tag in one SIMD compare. Each set bit is a candidate; the key compare runs only on those, which is rare for a miss. If the group has an `EMPTY` slot the key is absent; otherwise probing moves to the next group by triangular offsets (1, 3, 6, ...), which visits every group of a power-of-two map.

Load limit is 7/8 of the slots. `growth_left` counts empty slots that may still be filled. Inserting into a `DELETED` slot does not spend it, which is how delete-heavy workloads stay put. Removing marks a slot `EMPTY` when its group already has an empty slot (no probe ever went past that group), and `DELETED` otherwise. When `growth_left` reaches zero the map is rebuilt: at the same size if at most half the load limit is live (this drops the tombstones), otherwise at double. Rebuilding allocates a new block and reinserts, so it is not in place and briefly needs both blocks. The new block starts empty and only fills, so each entry's slot comes from a per-group fill count (a stack array, or a small allocation above 512 groups) instead of a vector load of the control bytes: entries of one old group land in the same one or two new groups, and a load right after a byte store to the same group has to wait for the store. The tests assert that insert/remove churn does not grow the capacity.

`for` iterates by group, taking each group's occupied-slot bits, visiting only the set bits (lowest first, highest first for `<`) and skipping empty groups whole. `remove it` clears the current slot the same way `delete` does; it never rebuilds, so the loop stays valid. Adding to the map inside the loop does not.

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
- Tests: `jacket test --path stdlib/Extensions/Hash_Map` (add `--interp`, or `-os wasm --runner "node --no-warnings tools/wasi_run.mjs"`, for the other modes). They cover basics, growth over many rebuilds, tombstone churn, string keys, custom hash and equal, iteration exactly once, removal during iteration, and the scalar/vector agreement.

## Configuration

- `SCALAR_GROUPS` (module parameter, default `false`): use plain loops for the group compares.
- `GROUP_SIZE :: 16` and the 7/8 load limit are constants in `module.jai`.
- `Map` parameters: `Key`, `Value`, optional `hash` `(Key) -> u32`, optional `equal` `(Key, Key) -> bool`. Defaults are `Hash.get_hash` (`hash_text` for strings) and `==`.
- The allocator is `map.allocator`, defaulting to `context.allocator` at first use.

## Dependencies

`Basic`, `Hash` (default hash) and `Extensions/SIMD`.

## Measured against Hash_Table and the community map

Hash_Map next to the standard library's `Hash_Table` and [`666rayen999--hash_map`](https://github.com/666rayen999/hash_map), the only other hash-map module in the upstream corpus. The cells come from reading each library's source; ⚠ has a note.

| Feature | Hash_Map | Hash_Table | rayen999 hash_map |
| --- | --- | --- | --- |
| Layout | one allocation: control bytes, then `key, value` entries; 16 slots compared per step | one array of `hash, key, value` entries, linear probing | three arrays inside the struct (`control`, `keys`, `values`), Robin Hood probing |
| Grows | ✅ at 7/8 load | ✅ at 70% load | ❌ [1] |
| Capacity | any, from 16 slots | any, from 32 | fixed per type (`$C`, a power of two) |
| Keys | any type, optional hash and equality procedures | any type, optional hash and equality procedures | ⚠ [2] |
| Insert, replace | ✅ returns the old value | ✅ | ⚠ [3] |
| Lookup | ✅ pointer or `null`, `get` with a default | ✅ | ❌ [4] |
| Remove | ✅ slots are reused | ✅ slots are reused (`REFILL_REMOVED`) | ❌ [5] |
| Remove while iterating | ✅ `remove it` | ✅ | ❌ |
| Iterate | ✅ forward, reverse, by pointer | ✅ | ⚠ forward, by value only |
| Entry that is created on first use | ✅ `entry` | ✅ `find_or_add` | ❌ |
| Reserve, clear | ✅ | ✅ | ❌ |
| Custom allocator | ✅ | ✅ | not applicable |
| Builds under jaic | ✅ | ✅ | ⚠ [6] |
| Tests that compare it with another map | ✅ `tests/agreement.jai`, also the scalar and vector paths | ✅ | ❌ |

1. `insert` returns `false` when it has probed every slot, and nothing makes room. The tables live in the struct, so `Hash_Map(string, float, 256)` is on the stack.
2. Enums, integers and strings only (`Is_Valid_Type`), hashed with the module's own XXH3.
3. `insert` swaps entries on the way (Robin Hood) but writes the new entry's probe distance into the control byte only when it stops at an empty slot. After a swap the slot's control byte keeps the previous owner's distance.
4. `get` ends a probe when the distance exceeds the control byte, so it misses a key that was swapped into a slot. Probing 200 random `s64` keys into 256 slots, 38 of them were not found again (162 of 200 were).
5. `delete` marks the slot `DELETED`, and `insert` only reuses `EMPTY` slots. Inserting one key and deleting it again, a table of 256 slots reported full after 256 pairs.
6. `xxh3.jai` declares `xxh64 :: (count: u64, ptr: *u8, $$ seed: u64)` and jaic types the call's argument as `s64`, so the module does not compile unchanged ("argument of type `s64` does not match parameter type `u64`"). The checks of notes 3 to 5 were made on a copy with `$$` removed from that signature.

### How it was measured

`benchmarks/compare/hash-map/benches/hash_map.jai`, run by `tools/bench/library_compare.py hash-map --per-item` (see [library comparisons](../tools/library-comparisons.md)). The community map is not measured: it returns wrong answers (notes 3 to 5), so a speed would be for a different job. Keys are random `s64` values (a fixed xorshift) or strings of the form `key-N-M` that live in one buffer, at 1,000, 100,000 and 1,000,000 entries.

- **insert**: a fresh table, every key inserted, growth included. **string_insert**: the same with string keys.
- **hit**, **miss**: lookups of keys in the table / not in it, after it was built before the timer started; hits in a shuffled order. **string_hit** is the same for strings.
- **churn**: a sliding window of constant size: each step removes the oldest key and inserts a new one.
- **iterate**: the sum of all values.

An iteration is one pass over all entries; the table shows the time per entry. Results go through `black_box`. Allocation is the default allocator in both.

Apple M5 (10 cores: 4 Super and 6 Efficiency), 16 GB, macOS 27.0 arm64. The machine was a working desktop, not an idle one: the load average when the run began was 3.43, so absolute times carry a few percent of noise (the intervals show the part that was measured within one run) and the ratios are the result to read.

Time per operation with the 95% interval, and the ratio of Hash_Table's time to Hash_Map's (above 1: Hash_Map is faster).

| Workload | hash_map | hash_table |
| --- | --- | --- |
| insert_1k | 13.4 ns (13.4 ns to 13.4 ns) | 12.2 ns (12.2 ns to 12.2 ns) 0.91x |
| insert_100k | 12.7 ns (12.7 ns to 12.7 ns) | 24.6 ns (24.6 ns to 24.7 ns) 1.94x |
| insert_1m | 16.3 ns (16.2 ns to 16.4 ns) | 35.2 ns (35 ns to 35.5 ns) 2.16x |
| hit_1k | 4.89 ns (4.88 ns to 4.9 ns) | 6.59 ns (6.57 ns to 6.6 ns) 1.35x |
| hit_100k | 6.51 ns (6.5 ns to 6.53 ns) | 7.3 ns (7.29 ns to 7.31 ns) 1.12x |
| hit_1m | 17.7 ns (17.5 ns to 17.9 ns) | 18.6 ns (18.5 ns to 18.7 ns) 1.05x |
| miss_1k | 5.05 ns (5.04 ns to 5.06 ns) | 5.76 ns (5.74 ns to 5.77 ns) 1.14x |
| miss_100k | 9.28 ns (9.27 ns to 9.28 ns) | 11.6 ns (11.5 ns to 11.6 ns) 1.25x |
| miss_1m | 6.5 ns (6.38 ns to 6.61 ns) | 25.6 ns (25.4 ns to 25.8 ns) 3.95x |
| churn_1k | 18.3 ns (18.2 ns to 18.3 ns) | 17.7 ns (17.5 ns to 17.7 ns) 0.97x |
| churn_100k | 19.5 ns (19.4 ns to 19.5 ns) | 29.7 ns (29.5 ns to 29.9 ns) 1.53x |
| churn_1m | 37.9 ns (37.2 ns to 38.7 ns) | 60.8 ns (60.2 ns to 61.3 ns) 1.60x |
| iterate_1k | 0.575 ns (0.574 ns to 0.575 ns) | 1.12 ns (1.11 ns to 1.12 ns) 1.94x |
| iterate_100k | 0.853 ns (0.851 ns to 0.856 ns) | 5.28 ns (5.25 ns to 5.3 ns) 6.18x |
| iterate_1m | 1.83 ns (1.82 ns to 1.83 ns) | 4.76 ns (4.73 ns to 4.79 ns) 2.61x |
| string_insert_1k | 18.9 ns (18.9 ns to 19 ns) | 22.6 ns (22.5 ns to 22.7 ns) 1.19x |
| string_insert_100k | 19.8 ns (19.6 ns to 19.9 ns) | 39 ns (38.8 ns to 39.1 ns) 1.97x |
| string_insert_1m | 34.3 ns (34.1 ns to 34.5 ns) | 56 ns (55.7 ns to 56.5 ns) 1.63x |
| string_hit_1k | 12.9 ns (12.9 ns to 12.9 ns) | 15.5 ns (15.5 ns to 15.5 ns) 1.20x |
| string_hit_100k | 18.7 ns (18.6 ns to 18.8 ns) | 21.3 ns (21.2 ns to 21.4 ns) 1.14x |
| string_hit_1m | 38.4 ns (38.1 ns to 38.7 ns) | 43.7 ns (43.4 ns to 44 ns) 1.14x |

Reading it: Hash_Map wins most where the map outgrows the cache (a miss ends after one group instead of walking a cluster; removed slots are reused; iteration skips empty groups). At 1,000 entries insert and churn are close to even: a small map rebuilds eight times on the way up (16 slots to 2,048) and every rebuild rehashes every key, where `Hash_Table` keeps the hash in its entries. Hits on a large string map are a little faster than `Hash_Table`'s although a lookup touches the control byte, the entry and then the string, where `Hash_Table` reads the entry and the string; the string hash reads eight bytes per step.
